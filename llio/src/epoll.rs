extern crate libc as c;

use std;
use std::io::{Error,Result};
use std::vec::Vec;
use super::RawFd;

use super::{EventType,Token};

pub struct Selector {
    fd : c::c_int,
}

pub struct Event {
    events: u32,
    token: Token,
}

pub struct Events {
    events: Vec<c::epoll_event>
}

impl Event {
    pub fn get_token(&self) -> Token {
        self.token
    }

    pub fn readable(&self) -> bool {
        (self.events & c::EPOLLIN as u32) != 0
    }

    pub fn writeable(&self) -> bool {
        (self.events & c::EPOLLOUT as u32) != 0
    }
}

impl Events {
    pub fn with_capacity(size: usize) -> Self {
        Events{ events: Vec::with_capacity(size) }
    }
}

pub struct EventIterator<'a> {
    itr : std::slice::Iter<'a, c::epoll_event>
}

impl <'a> Iterator for EventIterator<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.itr.next().map( |event| {
            Event{token: Token(event.u64), events: event.events}
        } )
    }
}

impl<'a> IntoIterator for &'a Events {
    type Item = Event;
    type IntoIter = EventIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        EventIterator{ itr: (&self.events).into_iter()}
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

    fn epoll_ctl(&mut self, op: c::c_int, fd: RawFd, event: &mut c::epoll_event) -> Result<()> {
        let res = unsafe { c::epoll_ctl(self.fd, op, fd, event) };
        super::to_void_result(res)
    }
    
    pub fn register(&mut self, token: Token, ty: EventType, fd: RawFd) -> Result<()> {
        let event_type = from_event_type(ty);
        let mut event = c::epoll_event{events : event_type, u64: token.0};
        self.epoll_ctl(c::EPOLL_CTL_ADD, fd, &mut event)
    }

    pub fn modify(&mut self, token: Token, ty: EventType, fd: RawFd) -> Result<()> {
        let event_type = from_event_type(ty);
        let mut event = c::epoll_event{events : event_type, u64: token.0};
        self.epoll_ctl(c::EPOLL_CTL_MOD, fd, &mut event)
    }

    pub fn unregister(&mut self, fd: self::RawFd) -> Result<()> {
        let mut event = c::epoll_event{events : 0, u64: 0};
        self.epoll_ctl(c::EPOLL_CTL_DEL, fd, &mut event)
    }
}
