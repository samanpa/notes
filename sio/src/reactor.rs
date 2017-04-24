use super::heaptimer;
use super::Context;
use super::error::{Error,Result};

use std;
use std::boxed::Box;
use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::vec::Vec;
use std::collections::HashMap;
use ::EventSource;

use llio::{Events,EventType,RawFd,Token,Selector};
use super::{Time,TimerTask};


enum ScheduledAction {
    Register(Token, EventType, Box<EventSource>),
    Modify(Token, EventType, Box<EventSource>),
    UnRegister(Token),
    Timer(Time, Box<TimerTask>)
}

pub struct Reactor {
    inner: Rc<RefCell<Inner>>,
    actions: Rc<RefCell<Vec<ScheduledAction>>>,
    run: bool,
}

pub struct Handle {
    inner: Weak<RefCell<Inner>>,
    actions: Weak<RefCell<Vec<ScheduledAction>>>
}

struct Event {
    fd: RawFd,
    source: Box<EventSource>
}

struct Inner {
    timer : heaptimer::HeapTimer,
    curr_token: u64,
    selector: Selector,
    events : HashMap<Token, Event>,
}

impl Inner {
    pub fn new() -> std::io::Result<Inner> {
        let timer = heaptimer::HeapTimer::new();
        let selector = try!(Selector::new());
        Ok(Inner{timer: timer
                 , curr_token: 1
                 , selector: selector
                 , events: HashMap::new()
        })
    }

    fn process(&mut self, ctx: &mut Context, token: Token) -> std::result::Result<(), (std::io::Error,RawFd)> {
        if let Some(ref mut event) = self.events.get_mut(&token) {
            return event.source.process(ctx)
                .map_err( |err| (err, event.fd) );
        }
        Ok(())
    }
    
    fn poll(&mut self, ctx: &mut Context, events: &mut Events, live: bool) {
        let _ = self.selector.poll(events, 1000_000_000);
        for event in events {
            let token = event.get_token();
            self.process(ctx, token)
                .map_err(|(err, fd)|{
                    use std::error::Error;
                    println!("ERROR fd[{}]: {}", fd, err.description());
                    let _ = self.unregister(token);
                });
        }
    }
    
    fn run_action(&mut self, action: ScheduledAction) -> std::io::Result<()> {
        match action {
            ScheduledAction::Register(token, ty, source) => {
                self.register(token, ty, source)
            },
            ScheduledAction::Modify(token, ty, source)=> {
                self.modify(token, ty, source)
            },
            ScheduledAction::UnRegister(token) => {
                self.unregister(token)
            },
            ScheduledAction::Timer(time, task) => {
                use super::Timer;
                self.timer.schedule(task, time);
                Ok(())
            }
        }
    }

    pub fn new_token(&mut self) -> Token {
        self.curr_token += 1;
        Token::new(self.curr_token - 1)
    }
    
    pub fn register(&mut self, token: Token, ty: EventType, source: Box<EventSource> ) -> std::io::Result<()> {
        let fd = source.fd();
        try!(self.selector.register(token, ty, fd));
        self.events.insert(token, Event{fd: fd, source: source});
        Ok(())
    }

    pub fn modify(&mut self, token: Token, ty: EventType, source: Box<EventSource> ) -> std::io::Result<()> {
        let fd = source.fd();
        match self.events.get_mut(&token) {
            None => return Err(Error::from_str("Invalid token")),
            Some(h) => {
                let _ = try!(self.selector.modify(token, ty, fd));
                *h = Event{fd: fd, source: source};
            }
        };
        Ok(())
    }

    pub fn unregister(&mut self, token: Token) -> Result<()> {
        let res = self.events.remove(&token).map( |Event{fd,source}| {
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
    pub fn new_token(&self) -> Result<Token> {
        match self.inner.upgrade() {
            None => Err(Error::from_str("Destroyed")),
            Some(inner) => Ok(inner.borrow_mut().new_token())
        }
    }
    
    pub fn register<H: 'static+EventSource>(&self, token: Token, ty: EventType, source: H) -> std::io::Result<()> {
        if let Some(actions) = self.actions.upgrade() {
            let reg = ScheduledAction::Register(token, ty, Box::new(source));
            actions.borrow_mut().push(reg);
            return Ok(())
        };
        Err(Error::from_str("Destroyed"))
    }

    pub fn unregister(&self, token: Token) -> std::io::Result<()> {
        if let Some(actions) = self.actions.upgrade() {
            let unreg = ScheduledAction::UnRegister(token);
            actions.borrow_mut().push(unreg);
            return Ok(())
        };
        Err(Error::from_str("Destroyed"))
    }

    pub fn modify<H: 'static+EventSource>(&mut self, token: Token, ty: EventType, source: H) -> std::io::Result<()> {
        if let Some(actions) = self.actions.upgrade() {
            let modify = ScheduledAction::Modify(token, ty, Box::new(source));
            actions.borrow_mut().push(modify);
            return Ok(())
        };
        Err(Error::from_str("Destroyed"))
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Handle{ inner: self.inner.clone()
                , actions: self.actions.clone() }
    }
}

impl Reactor {
    pub fn new() -> std::io::Result<Reactor> {
        let inner = try!(Inner::new());
        Ok(Reactor{inner: Rc::new(RefCell::new(inner))
                   ,run: false
                   , actions: Rc::new(RefCell::new(Vec::with_capacity(1)))})
    }

    pub fn stop(&mut self) {
        self.run = false
    }


    fn run_once(&mut self, ctx: &mut Context, events: &mut Events, live: bool) {
        let mut inner = self.inner.borrow_mut();
        { 
            let mut actions = self.actions
                .borrow_mut();
            for action in actions.drain(..) {
                inner.run_action(action);
            };
        };
        inner.poll(ctx, events, live);
    }

    //FIXME: return error
    pub fn run(&mut self, ctx: &mut Context, live: bool) {
        self.run = true;
        let mut events = Events::with_capacity(2);
        while self.run {
            self.run_once(ctx, &mut events, live);
        }
    }

    /**
     * self is not borrowed mutablly to allow this method to be called
     *    to be sent while we are in the middle of a poll
     **/
    fn send_action(&self, action: ScheduledAction) {
        self.actions
            .borrow_mut()
            .push(action);
    }
    

    pub fn handle(&self) -> Handle {
        Handle{ inner: Rc::downgrade(&self.inner),
                actions: Rc::downgrade(&self.actions)
        }
    }

    pub fn register<H: 'static+EventSource>(&mut self, token: Token, ty: EventType, source: H) -> std::io::Result<()> {
        self.inner.borrow_mut().register(token, ty, Box::new(source))
    }


    pub fn modify<H: 'static+EventSource>(&mut self, token: Token, ty: EventType, source: H) -> std::io::Result<()> {
        self.inner.borrow_mut().modify(token, ty, Box::new(source))
    }

    pub fn new_token(&self) -> Token {
        self.inner.borrow_mut().new_token()
    }
    
}
