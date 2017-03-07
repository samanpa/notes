use core;
use core::simpletimer;
use core::{Context,Time};
use core::error::{Error,Result};
use core::event::EventHandler;

use std;
use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::collections::HashMap;

use plat::net::{Events,EventType,Token,Selector};

pub struct Reactor {
    inner: Rc<RefCell<Inner>>
}

pub struct Handle{
    inner: Weak<RefCell<Inner>>
}

pub struct Inner {
    timer : simpletimer::SimpleTimer,
    run : bool,
    curr_token: u64,
    selector: Selector
}


impl Inner {
    pub fn new() -> Result<Inner> {
        let timer = simpletimer::SimpleTimer::new();
        let selector = match Selector::new() {
            Err(err) => return Err(Error::from_err(err)),
            Ok(selector) => selector
        };

        Ok(Inner{timer: timer, run: false, curr_token: 1, selector: selector})
    }


    fn run_once(&mut self, ctx: &mut Context, live: bool) {
        let mut events = Events::with_capacity(2);
        self.selector.select(&mut events, 1000_000);
        for event in &events {
            println!("token {:?}", Selector::get_token(&event))
        }
        
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
        let token = Token::new(self.curr_token - 1);
        match self.selector.register(token, ty, handler.fd()) {
            Ok(_)  => Ok(token),
            Err(e) => Err(Error::from_err(e))
        }
    }

    pub fn unregister(&mut self, token: Token) -> Result<()> {
        match self.selector.unregister(token) {
            Ok(_)  => Ok(()),
            Err(e) => Err(Error::from_err(e))
        }
    }
}

impl Handle {
    pub fn register(&self, ty: EventType, handler: Rc<EventHandler>) -> Result<Token> {
        if let Some(inner) = self.inner.upgrade() {
            return inner.borrow_mut().register(ty, handler);
        };
        Err(Error::from_str("Destroyed"))
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Handle{ inner: self.inner.clone() }
    }
}

impl Reactor {
    pub fn new() -> Result<Reactor> {
        let inner = try!(Inner::new());
        Ok(Reactor{inner: Rc::new(RefCell::new(inner))})
    }

    pub fn stop(&mut self) {
        self.inner.borrow_mut().stop();
    }


    pub fn handle(&self) -> Handle {
        Handle{ inner: Rc::downgrade(&self.inner) }
    }

    pub fn run(&mut self, ctx: &mut Context, live: bool) {
        self.inner.borrow_mut().run(ctx, live);
    }

    pub fn register(&mut self, ty: EventType, handler: Rc<EventHandler>) -> Result<Token> {
        self.inner.borrow_mut().register(ty, handler)
    }
}

