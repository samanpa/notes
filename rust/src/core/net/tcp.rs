extern crate libc as c;

use std;
use core;
use core::Timer;
use core::error::{Error,Result};
use core::event::*;
use std::rc::{Rc,Weak};
use std::cell::RefCell;

use super::socket::Socket;

pub enum TcpState {
    Connected,
    Connecting,
    Disconnected,
    NotInitialized,
    Closed
}

pub struct TcpStream {
    sock : Socket
}

pub struct TcpClient<T> 
    where T : Timer {
    inner: RefCell<Inner<T>>
}

pub struct Inner<T> 
    where T : Timer {
    token: Option<core::reactor::Token>,
    stream: TcpStream,
    reactor: core::reactor::Handle<T>,
    state: TcpState,
}

impl TcpStream {
    pub fn connect(addr: &std::net::SocketAddrV4) -> Result<TcpStream> {
        let mut socket = try!(Socket::new(c::AF_INET, c::SOCK_STREAM, 0));
        socket.nonblock();
        try!(socket.connect(addr));
        Ok(TcpStream{ sock: socket })
    }
}

impl EventHandler for TcpStream {
    fn process(&mut self, ctx: &mut core::Context) {
        println!("Handle event");
    }

    fn fd(&self) -> i32 {
        self.sock.fd() as i32
    }
}


impl <T> TcpClient<T> where T: Timer {
    //return an Rc seems wrong
    pub fn connect(addr: &std::net::SocketAddrV4, reactor: core::reactor::Handle<T> ) -> Result<Rc<Self>> {
        let stream = try!(TcpStream::connect(addr));
        let inner  = Inner { token : None,
                             stream: stream,
                             reactor: reactor.clone(),
                             state: TcpState::NotInitialized };
        let client = Rc::new(TcpClient{ inner: RefCell::new(inner) });
        let token = try!(reactor.register(EventType::Read, client.clone()));
        client.inner.borrow_mut().token = Some(token);
        Ok(client)
    }
}

impl <T> EventHandler for TcpClient<T> where T: Timer {
    fn process(&mut self, ctx: &mut core::Context) {
        self.inner.borrow_mut().stream.process(ctx);
    }

    fn fd(&self) -> i32 {
        self.inner.borrow_mut().stream.sock.fd() as i32
    }
}

