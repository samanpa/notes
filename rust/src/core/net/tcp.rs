extern crate libc as c;

use std;
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

pub fn connect(addr: &std::net::SocketAddrV4) -> Result<TcpStream> {
    let mut socket = try!(Socket::new(c::AF_INET, c::SOCK_STREAM, 0));
    socket.nonblock();
    let res = try!(socket.connect(addr));
    Ok(TcpStream{ sock: socket })
}
