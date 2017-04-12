use super::heaptimer;
use super::Context;
use super::error::{Error,Result};

use std;
use std::boxed::Box;
use std::rc::{Rc,Weak};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use ::EventHandler;

use llio::{Events,EventType,RawFd,Token,Selector};

pub struct Reactor {
    inner: Rc<UnsafeCell<Inner>>
}

pub struct Handle{
    inner: Weak<UnsafeCell<Inner>>
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
    pub fn new() -> std::io::Result<Inner> {
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
            let token = event.get_token();
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
    
    pub fn register<H:'static + EventHandler>(&mut self, token: Token, ty: EventType, handler: H ) -> std::io::Result<()> {
        let fd = handler.fd();
        self.events.insert(token, Event{fd: fd, handler: Box::new(handler)});
        self.selector.register(token, ty, fd)
    }

    pub fn modify<H: 'static + EventHandler>(&mut self, token: Token, ty: EventType, handler: H ) -> std::io::Result<()> {
        let fd = handler.fd();
        match self.events.get_mut(&token) {
            None => return Err(Error::from_str("Invalid token")),
            Some(h) => {
                let _ = try!(self.selector.modify(token, ty, fd));
                *h = Event{fd: fd, handler: Box::new(handler)};
            }
        };
        Ok(())
    }

    pub fn unregister(&mut self, token: Token) -> Result<()> {
        //Fixme: Move this to the event loop?
        let res = self.events.remove(&token).map( |Event{fd,handler}| {
            self.selector.unregister(fd )
        });
        match res {
            None         => Err(Error::from_str("Token not found")),
            Some(Ok(_))  => Ok(()),
            Some(e)      => e
        }
    }
}

impl Handle {
    fn get<'a>(inner: Rc<UnsafeCell<Inner>>) -> &'a mut Inner {
        unsafe{ &mut *inner.get() }
    }

    pub fn new_token(&self) -> Result<Token> {
        match self.inner.upgrade() {
            None => Err(Error::from_str("Destroyed")),
            Some(inner) => Ok(Handle::get(inner).new_token())
        }
    }
    
    pub fn register<H: 'static+EventHandler>(&self, token: Token, ty: EventType, handler: H) -> std::io::Result<()> {
        if let Some(inner) = self.inner.upgrade() {
            return Handle::get(inner).register(token, ty, handler);
        };
        Err(Error::from_str("Destroyed"))
    }

    pub fn modify<H: 'static+EventHandler>(&mut self, token: Token, ty: EventType, handler: H) -> std::io::Result<()> {
        if let Some(inner) = self.inner.upgrade() {
            return Handle::get(inner).modify(token, ty, handler);
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
    fn get_inner<'a>(&self) -> &'a mut Inner {
        unsafe{ &mut *self.inner.get() }
    }

    pub fn new() -> std::io::Result<Reactor> {
        let inner = try!(Inner::new());
        Ok(Reactor{inner: Rc::new(UnsafeCell::new(inner))})
    }

    pub fn stop(&mut self) {
        self.get_inner().stop();
    }


    pub fn handle(&self) -> Handle {
        Handle{ inner: Rc::downgrade(&self.inner) }
    }

    pub fn run(&mut self, ctx: &mut Context) {
        self.get_inner().run(ctx, true);
    }

    pub fn register<H: 'static+EventHandler>(&mut self, token: Token, ty: EventType, handler: H) -> std::io::Result<()> {
        self.get_inner().register(token, ty, handler)
    }


    pub fn modify<H: 'static+EventHandler>(&mut self, token: Token, ty: EventType, handler: H) -> std::io::Result<()> {
        self.get_inner().modify(token, ty, handler)
    }
}
