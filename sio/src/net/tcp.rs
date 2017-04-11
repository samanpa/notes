extern crate libc as c;
extern crate llio;

use std;
use llio::{EventType,Token,TcpStream};
use ::error::Result;
use ::reactor::EventHandler;
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
    inner: Weak<RefCell<Inner>>,
    fd: llio::RawFd
}

struct Inner {
    token: Token,
    stream: TcpStream,
    reactor: ::reactor::Handle,
    state: TcpState,
}


impl TcpClient {
    //return an Rc seems wrong
    pub fn connect(addr: &std::net::SocketAddrV4, reactor: ::reactor::Handle ) -> Result<Self> {
        let stream = try!(TcpStream::new());
        try!(stream.nonblock());
        try!(stream.connect(addr));
        let token = try!(reactor.new_token());
        let fd = stream.fd();
        let inner  = Inner { token: token,
                             stream: stream,
                             reactor: reactor.clone(),
                             state: TcpState::NotInitialized };
        let inner = Rc::new(RefCell::new(inner));
        let handle = TcpHandle{ inner: Rc::downgrade(&inner), fd: fd };
        try!(reactor.register(token, EventType::Write, Box::new(handle)));
        let client = TcpClient{ inner: inner };
        Ok(client)
    }
}

impl EventHandler for Inner
{
    fn process(&mut self, ctx: &mut ::Context) {
        println!("Handle event");
    }

    fn fd(&self) -> llio::RawFd {
        self.stream.fd()
    }

}

impl EventHandler for TcpHandle
{
    fn process(&mut self, ctx: &mut ::Context) {
        self.inner.upgrade()
            .map( |inner| inner.borrow_mut().process(ctx) );
    }

    fn fd(&self) -> llio::RawFd {
        self.fd
    }
}

impl EventHandler for TcpClient {
    fn process(&mut self, ctx: &mut ::Context) {
        self.inner.borrow_mut().process(ctx)
    }

    fn fd(&self) -> llio::RawFd {
        self.inner.borrow().fd()
    }
}
