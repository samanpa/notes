extern crate libc as c;

use std;
use core;
use core::Timer;
use plat::net::{EventType,Token,Socket};
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

pub struct TcpStream {
    sock : Socket
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

impl TcpStream {
    pub fn connect(addr: &std::net::SocketAddrV4) -> Result<TcpStream> {
        let mut socket = match Socket::new(c::AF_INET, c::SOCK_STREAM, 0) {
            Ok(socket) => socket,
            Err(e)     => return Err(Error::from_err(e))
        };
        socket.nonblock();
        match socket.connect(addr) {
            Ok(_) => Ok(TcpStream{ sock: socket }),
            Err(e) => Err(Error::from_err(e))
        }
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


impl TcpClient {
    //return an Rc seems wrong
    pub fn connect(addr: &std::net::SocketAddrV4, reactor: core::reactor::Handle ) -> Result<Rc<Self>> {
        let stream = try!(TcpStream::connect(addr));
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
        self.inner.borrow_mut().stream.process(ctx);
    }

    fn fd(&self) -> i32 {
        self.inner.borrow_mut().stream.sock.fd() as i32
    }
}

