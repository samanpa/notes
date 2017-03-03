extern crate libc as c;

use super::{Context, Time, Timer};
use core;
use core::error::{Error,Result};
use core::event::{EventType,EventHandler};
use std;
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Copy,Clone)]
pub struct Token(u64);

impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::cmp::PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        return self.0 == other.0
    }
}
impl std::cmp::Eq for Token {}


pub struct Reactor<F>
    where F : Timer {
    timer : F,
    fd : c::c_int,
    run : bool,
    curr_token: u64,
    events : HashMap<Token, Rc<EventHandler>> //Seems slow
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

impl<T> Reactor<T> where T : Timer {
    pub fn new(timer: T) -> Result<Reactor<T>> {
        let fd = unsafe{ c::epoll_create(c::EPOLL_CLOEXEC) };
        if fd == -1 {
            return Err(Error::from_str("Could not create epoll fd"))
        }

        Ok(Reactor{timer: timer, fd: fd, run: false, curr_token: 0, events: HashMap::new()})
    }


    fn run_once(&mut self, ctx: &mut Context, live: bool) {
        let mut event = c::epoll_event{ events: 0, u64: 0 };
        let max_events: c::c_int = 1;
        let timeout: c::c_int  = 1000;
        unsafe { c::epoll_wait(self.fd
                               , &mut event as *mut c::epoll_event
                               , max_events
                               , timeout)
        };
    }

    fn stop(&mut self) {
        self.run = false
    }
    

    pub fn run(&mut self, ctx: &mut Context, live: bool) {
        self.run = true;
        while self.run {
            self.run_once(ctx, live);
        }
    }

    pub fn register(&mut self, ty: EventType, handler: Rc<EventHandler>) -> Result<Token> {
        self.curr_token += 1;
        let token = Token(self.curr_token - 1);
        let event_type = from_event_type(ty);
        let mut event = c::epoll_event{events : event_type, u64: token.0};
        let res = unsafe {
            c::epoll_ctl(self.fd, c::EPOLL_CTL_ADD, handler.fd(), &mut event)
        };
        match core::to_result(res) {
            Ok(_)  => {
                self.events.insert(token, handler);
                Ok(token)
            }
            Err(e) => Err(Error::from_str(std::error::Error::description(&e)))
        }
    }
}

impl<T> Drop for Reactor<T>  where T : Timer {
    fn drop(&mut self) {
        unsafe{ c::close(self.fd); }
    }
}
