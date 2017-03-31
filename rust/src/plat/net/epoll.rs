extern crate libc as c;

use std;
use std::io::{Error,Result};
use std::vec::Vec;

use super::{EventType,Token};

pub struct Selector {
    fd : c::c_int,
}

pub type Event = c::epoll_event;
pub struct Events {
    events: Vec<Event>
}

impl Events {
    pub fn with_capacity(size: usize) -> Self {
        Events{ events: Vec::with_capacity(size) }
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
            fd  => Ok(Selector{ fd: fd })
        }
    }

    pub fn get_token(event : &Event) -> Token {
        Token(event.u64)
    }
    
    pub fn poll(&mut self, events: &mut Events, timeout_ns: u64) -> Result<()> {
        let max_events = events.events.capacity() as c::c_int;
        let res = unsafe { c::epoll_wait(self.fd
                                         , events.events.as_mut_ptr()
                                         , max_events
                                         , (timeout_ns / 1000_000) as c::c_int)
        };
        if res > 0 {
            unsafe{ events.events.set_len(res as usize) };
        }
        super::to_void_result(res)
    }

    pub fn register(&mut self, token: Token, ty: EventType, fd: c::c_int) -> Result<()> {
        let event_type = from_event_type(ty);
        let mut event = c::epoll_event{events : event_type, u64: token.0};
        let res = unsafe {
            c::epoll_ctl(self.fd, c::EPOLL_CTL_ADD, fd, &mut event)
        };
        super::to_void_result(res)
    }

    pub fn unregister(&mut self, fd: c::c_int) -> Result<()> {
        let mut event = c::epoll_event{events : 0, u64: 0};
        let res = unsafe {
            c::epoll_ctl(self.fd, c::EPOLL_CTL_DEL, fd, &mut event)
        };
        super::to_void_result(res)
    }
}
