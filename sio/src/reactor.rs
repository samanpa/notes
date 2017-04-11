use super::heaptimer;
use super::Context;
use super::error::{Error,Result};

use std::boxed::Box;
use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::collections::HashMap;

use llio::{Events,EventType,RawFd,Token,Selector};

pub trait EventHandler {
    fn process(&mut self, ctx: &mut super::Context);
    fn fd(&self) -> ::llio::RawFd;
}

pub struct Reactor {
    inner: Rc<RefCell<Inner>>
}

pub struct Handle{
    inner: Weak<RefCell<Inner>>
}

struct Event {
    fd: RawFd,
    handler: Box<EventHandler>
}

struct Inner {
    timer : heaptimer::HeapTimer,
    run : bool,
    curr_token: u64,
    selector: Selector,
    events : HashMap<Token, Event>,
}

impl Inner {
    pub fn new() -> Result<Inner> {
        let timer = heaptimer::HeapTimer::new();
        let selector = try!(Selector::new());
        Ok(Inner{timer: timer
                 , run: false
                 , curr_token: 1
                 , selector: selector
                 , events: HashMap::new()})
    }


    fn run_once(&mut self, ctx: &mut Context, live: bool) {
        let mut events = Events::with_capacity(2);
        let _ = self.selector.poll(&mut events, 1000_000);
        for event in &events {
            let token = Selector::get_token(&event);
            self.events.get_mut(&token)
                .map( |ref mut event| {
                    event.handler.process(ctx)
                } );
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

    pub fn new_token(&mut self) -> Token {
        self.curr_token += 1;
        Token::new(self.curr_token - 1)
    }
    
    pub fn register(&mut self, token: Token, ty: EventType, handler: Box<EventHandler>) -> Result<()> {
        let fd = handler.fd();
        self.events.insert(token, Event{fd: fd, handler: handler});
        match self.selector.register(token, ty, fd) {
            Ok(_)  => Ok(()),
            Err(e) => Err(Error::from(e))
        }
    }

    pub fn unregister(&mut self, token: Token) -> Result<()> {
        //Fixme: Move this to the event loop?
        let res = self.events.remove(&token).map( |Event{fd,handler}| {
            self.selector.unregister(fd )
        });
        match res {
            None         => Err(Error::from_str("Token not found")),
            Some(Ok(_))  => Ok(()),
            Some(Err(e)) => Err(Error::from(e))
        }
    }
}

impl Handle {
    pub fn new_token(&self) -> Result<Token> {
        match self.inner.upgrade() {
            None => Err(Error::from_str("Destroyed")),
            Some(inner) => Ok(inner.borrow_mut().new_token())
        }
    }
    
    pub fn register(&self, token: Token, ty: EventType, handler: Box<EventHandler>) -> Result<()> {
        if let Some(inner) = self.inner.upgrade() {
            return inner.borrow_mut().register(token, ty, handler);
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

    pub fn register(&mut self, token: Token, ty: EventType, handler: Box<EventHandler>) -> Result<()> {
        self.inner.borrow_mut().register(token, ty, handler)
    }
}
