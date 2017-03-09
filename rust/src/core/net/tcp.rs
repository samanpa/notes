extern crate libc as c;

use std;
use core;
use core::Timer;
use plat::net::{EventType,Token,Socket,TcpStream};
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
    inner: RefCell<Inner>
}

pub struct Inner {
    token: Option<Token>,
    stream: TcpStream,
    reactor: core::reactor::Handle,
    state: TcpState,
}

impl TcpClient {
    //return an Rc seems wrong
    pub fn connect(addr: &std::net::SocketAddrV4, reactor: core::reactor::Handle ) -> Result<Rc<Self>> {
        let stream = try!(TcpStream::new());
        try!(stream.nonblock());
        try!(stream.connect(addr));
        let inner  = Inner { token : None,
                             stream: stream,
                             reactor: reactor.clone(),
                             state: TcpState::NotInitialized };
        let client = Rc::new(TcpClient{ inner: RefCell::new(inner) });
        let token = try!(reactor.register(EventType::Write, client.clone()));
        client.inner.borrow_mut().token = Some(token);
        Ok(client)
    }
}

impl EventHandler for TcpClient {
    fn process(&mut self, ctx: &mut core::Context) {
        println!("Handle event");
    }

    fn fd(&self) -> i32 {
        self.inner.borrow().stream.fd() as i32
    }
}

