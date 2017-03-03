extern crate libc as c;

use std;
use core;
use core::error::{Error,Result};
use core::event::*;
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

impl TcpStream {
    pub fn connect(addr: &std::net::SocketAddrV4) -> Result<TcpStream> {
        let mut socket = try!(Socket::new(c::AF_INET, c::SOCK_STREAM, 0));
        socket.nonblock();
        let res = try!(socket.connect(addr));
        Ok(TcpStream{ sock: socket })
    }
}

impl EventHandler for TcpStream {
    fn process(&mut self, ctx: &mut core::Context) {
    }

    fn fd(&self) -> i32 {
        self.sock.fd() as i32
    }
}
