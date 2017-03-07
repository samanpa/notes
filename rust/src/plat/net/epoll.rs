extern crate libc as c;

use std;
use std::collections::HashMap;
use std::io::{Error,ErrorKind,Result};
use std::vec::Vec;

use super::{EventType,Token,Socket};

pub struct Selector {
    fd : c::c_int,
    events : HashMap<Token, c::c_int> //Seems slow
}

pub type Event = c::epoll_event;
pub struct Events {
    events: std::vec::Vec<c::epoll_event>
}

impl Events {
    pub fn with_capacity(size: usize) -> Self {
        Events{ events: std::vec::Vec::with_capacity(size) }
    }
}

impl<'a> IntoIterator for &'a Events {
    type Item = &'a Event;
    type IntoIter = std::slice::Iter<'a, Event>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.events).into_iter()
    }

}


fn from_event_type(ty: EventType) -> u32
{
    let res = match ty {
        EventType::ReadWrite => c::EPOLLIN | c::EPOLLOUT,
        EventType::Read      => c::EPOLLIN,
        EventType::Write     => c::EPOLLOUT,
    };

    res as u32
}

impl Selector {
    pub fn new() -> Result<Selector> {
        let fd = unsafe{ c::epoll_create(c::EPOLL_CLOEXEC) };
        match fd {
            -1  => Err(Error::last_os_error()),
            fd  => Ok(Selector{ fd: fd, events: HashMap::new() })
        }
    }

    pub fn get_token(event : &Event) -> Token {
        Token(event.u64)
    }
    
    pub fn select(&mut self, events: &mut Events, timeout_ns: u64) -> Result<()> {
        let max_events = events.events.capacity() as c::c_int;
        let res = unsafe { c::epoll_wait(self.fd
                                         , events.events.as_mut_ptr()
                                         , max_events
                                         , (timeout_ns / 1000_000) as c::c_int)
        };
        if res > 0 {
            unsafe{ events.events.set_len(res as usize) };
        }
        super::to_result(res)            
    }

    pub fn register(&mut self, token: Token, ty: EventType, fd: c::c_int) -> Result<()> {
        let event_type = from_event_type(ty);
        let mut event = c::epoll_event{events : event_type, u64: token.0};
        let res = unsafe {
            c::epoll_ctl(self.fd, c::EPOLL_CTL_ADD, fd, &mut event)
        };
        let _ = try!(super::to_result(res));
        self.events.insert(token, fd);
        Ok(())
    }

    pub fn unregister(&mut self, token: Token) -> Result<()> {
        let res = self.events.remove(&token).map( |fd| {
            let mut event = c::epoll_event{events : 0, u64: 0};
            let res = unsafe {
                c::epoll_ctl(self.fd, c::EPOLL_CTL_DEL, fd, &mut event)
            };
            super::to_result(res)
        });
        match res {
            None => Err(Error::new(ErrorKind::Other, "Token not found")),
            Some(e) => e
        }
    }
}
