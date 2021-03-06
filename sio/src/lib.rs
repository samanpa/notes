pub mod time;
pub mod heaptimer;
pub mod simpletimer;
pub mod error;
pub mod reactor;
pub mod service;
pub mod net;

pub use self::time::Time;
pub use service::Service;

extern crate libc as c;
extern crate llio;

#[allow(dead_code)]
pub struct Context {
    event_time : Time,
    exchange_time : Time,
    channel : u64,
}

impl Context {
    pub fn new(channel: u64) -> Self {
        Context{ event_time : Time::now(),
                 exchange_time: Time::now(),
                 channel: channel }
    }
}

pub trait TimerTask {
    fn run(&mut self, ctx: &Context, time: Time);
}

pub trait Timer {
    fn schedule(&mut self, task: Box<TimerTask>, time: Time);
    fn process(&mut self, ctx: &Context, time: Time);
    fn peek_time(&self) -> Option<Time>;
}

pub enum Async<T> {
    Ready(T),
    NotReady
}    

impl<T> Async<T> {
    /// Change the success type of this `Async` value with the closure provided
    pub fn map<F, U>(self, f: F) -> Async<U>
        where F: FnOnce(T) -> U
    {
        match self {
            Async::Ready(t) => Async::Ready(f(t)),
            Async::NotReady => Async::NotReady,
        }
    }

    /// Returns whether this is `Async::Ready`
    pub fn is_ready(&self) -> bool {
        match *self {
            Async::Ready(_) => true,
            Async::NotReady => false,
        }
    }

    /// Returns whether this is `Async::NotReady`
    pub fn is_not_ready(&self) -> bool {
        !self.is_ready() 
    }
} 


pub trait EventSource {
    fn process(&mut self, ctx: &mut Context) -> std::io::Result<()>;
    fn fd(&self) -> llio::RawFd;
}
