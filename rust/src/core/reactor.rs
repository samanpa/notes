extern crate libc as c;

use core;
use core::{Context, Time, Timer};
use core::error::{Error,Result};
use core::event::{EventType,EventHandler};

use std;
use std::rc::{Rc,Weak};
use std::cell::RefCell;
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
    inner: Rc<RefCell<Inner<F>>>
}

pub struct Handle<F>
    where F : Timer {
    inner: Weak<RefCell<Inner<F>>>
}

pub struct Inner<F>
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

impl<T> Inner<T> where T : Timer {
    pub fn new(timer: T) -> Result<Inner<T>> {
        let fd = unsafe{ c::epoll_create(c::EPOLL_CLOEXEC) };
        if fd == -1 {
            return Err(Error::from_str("Could not create epoll fd"))
        }

        Ok(Inner{timer: timer, fd: fd, run: false, curr_token: 1, events: HashMap::new()})
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
            },
            Err(e) => Err(Error::from_err(e))
        }
    }

    pub fn unregister(&mut self, token: Token) -> Result<()> {
        let result = self.events.remove(&token).map( |handler| {
            let mut event = c::epoll_event{events : 0, u64: 0};
            let res = unsafe {
                c::epoll_ctl(self.fd, c::EPOLL_CTL_DEL, handler.fd(), &mut event)
            };
            match core::to_result(res) {
                Ok(_)  => Ok(()),
                Err(e) => Err(Error::from_err(e))
            }
        });
        match result {
            None    => Err(Error::from_str("Token not registerd")),
            Some(r) => r
        }
    }
}

impl<T> Handle<T> where T: Timer {
    pub fn register(&mut self, ty: EventType, handler: Rc<EventHandler>) -> Result<Token> {
        if let Some(inner) = self.inner.upgrade() {
            return inner.borrow_mut().register(ty, handler);
        };
        Err(Error::from_str("Destroyed"))
    }
}

impl<T> Clone for Handle<T> where T : Timer {
    fn clone(&self) -> Self {
        Handle{ inner: self.inner.clone() }
    }
}

impl<T> Reactor<T> where T : Timer {
    pub fn new(timer: T) -> Result<Reactor<T>> {
        let inner = try!(Inner::new(timer));
        Ok(Reactor{inner: Rc::new(RefCell::new(inner))})
    }

    pub fn stop(&mut self) {
        self.inner.borrow_mut().stop();
    }


    pub fn handle(&self) -> Handle<T> {
        Handle{ inner: Rc::downgrade(&self.inner) }
    }

    pub fn run(&mut self, ctx: &mut Context, live: bool) {
        self.inner.borrow_mut().run(ctx, live);
    }

    pub fn register(&mut self, ty: EventType, handler: Rc<EventHandler>) -> Result<Token> {
        self.inner.borrow_mut().register(ty, handler)
    }
}

