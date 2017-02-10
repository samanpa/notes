extern crate libc;

use super::{Context, Time, Timer};
use core::error::Error;
use core::event::{EventType,EventHandler};
use std;
use std::rc::Rc;
use std::collections::HashMap;
use std::result::Result;

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
    fd : libc::c_int,
    run : bool,
    curr_token: u64,
    events : HashMap<Token, Rc<EventHandler>> //Seems slow
}


fn from_event_type(ty: EventType) -> u32
{
    let res = match ty {
        EventType::ReadWrite => libc::EPOLLIN | libc::EPOLLOUT,
        EventType::Read      => libc::EPOLLIN,
        EventType::Write     => libc::EPOLLOUT,
    };

    res as u32
}

impl<T> Reactor<T> where T : Timer {
    pub fn new(timer: T) -> Result<Reactor<T>,Error> {
        let fd = unsafe{ libc::epoll_create(libc::EPOLL_CLOEXEC) };
        if fd == -1 {
            return Err(Error::from_str("Could not create epoll fd"))
        }

        Ok(Reactor{timer: timer, fd: fd, run: false, curr_token: 0, events: HashMap::new()})
    }


    fn run_once(&mut self, ctx: &mut Context, live: bool) {
    }

    fn stop(&mut self) {
        self.run = false
    }
    

    pub fn run(&mut self, ctx: &mut Context, live: bool) {
        while self.run {
            self.run_once(ctx, live)
        }
    }

    fn add(&mut self, ty: EventType, handler: Rc<EventHandler>) -> Option<Token> {
        self.curr_token += 1;
        let token = Token(self.curr_token - 1);
        let event_type = from_event_type(ty);
        let mut event = libc::epoll_event{events : event_type, u64: token.0};
        unsafe {
            libc::epoll_ctl(self.fd, libc::EPOLL_CTL_ADD, handler.get_fd(), &mut event);
        }
        Some(token)
    }
}

impl<T> Drop for Reactor<T>  where T : Timer {
    fn drop(&mut self) {
        //libc::close(self.fd)
    }
}
