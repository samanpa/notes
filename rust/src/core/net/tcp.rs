extern crate libc as c;

use std;
use core::error::Result;
use core::event::*;
use super::socket::Socket;
use super::addr;

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
    let socket = try!(Socket::new(c::AF_INET, c::SOCK_STREAM, 0));
    let result = addr::connect(&socket, addr);
    let stream = TcpStream{ sock: socket };
    return Ok(stream)
}
