extern crate libc as c;

use std;
use core;
use core::Timer;
use plat::net::{EventType,Token,TcpStream};
use core::error::{Error,Result};
use core::event::*;
use std::rc::{Rc,Weak};
use std::cell::RefCell;

pub enum TcpState {
    Connected,
    Connecting,
    Disconnected,
    NotInitialized,
    Closed
}

pub struct TcpClient {
    inner: Rc<RefCell<Inner>>
}

pub struct TcpHandle {
    inner: Weak<RefCell<Inner>>
}

pub struct Inner {
    token: Token,
    stream: TcpStream,
    reactor: core::reactor::Handle,
    state: TcpState,
}

impl TcpClient {
    //return an Rc seems wrong
    pub fn connect(addr: &std::net::SocketAddrV4, reactor: core::reactor::Handle ) -> Result<Self> {
        let stream = try!(TcpStream::new());
        try!(stream.nonblock());
        try!(stream.connect(addr));
        let token = try!(reactor.new_token());
        let inner  = Inner { token: token,
                             stream: stream,
                             reactor: reactor.clone(),
                             state: TcpState::NotInitialized };
        let inner = Rc::new(RefCell::new(inner));
        let weak = Rc::downgrade(&inner);
        let token = try!(reactor.register(token, EventType::Write, weak));
        let client = TcpClient{ inner: inner };
        Ok(client)
    }
}

impl EventHandler for Inner
{
    fn process(&mut self, ctx: &mut core::Context) {
        println!("Handle event");
    }

    fn fd(&self) -> i32 {
        self.stream.fd()
    }

}
impl EventHandler for RefCell<Inner>
{
    fn process(&mut self, ctx: &mut core::Context) {
        self.borrow_mut().process(ctx)
    }

    fn fd(&self) -> i32 {
        self.borrow().fd()
    }
}

impl EventHandler for TcpClient {
    fn process(&mut self, ctx: &mut core::Context) {
        self.inner.borrow_mut().process(ctx)
    }

    fn fd(&self) -> i32 {
        self.inner.fd()
    }
}

