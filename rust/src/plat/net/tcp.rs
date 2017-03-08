extern crate libc as c;

use std::io::{Error,ErrorKind,Result};
use std::net::{Ipv4Addr,SocketAddrV4};
use std;

pub struct TcpStream {
    socket : super::Socket,
    addr   : SocketAddrV4,
}

impl TcpStream {
    pub fn new(socket: super::Socket, addr: SocketAddrV4) -> Self {
        TcpStream{ socket: socket, addr: addr }
    }

    pub fn fd(&self) -> super::RawFd {
        self.socket.fd()
    }    
}


impl std::io::Read for TcpStream {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize> {
        let len = unsafe{ c::read(self.socket.fd()
                                  , buff.as_mut_ptr() as *mut c::c_void
                                  , buff.len()) };
        match len {
            len if len < 0 => Err(Error::last_os_error()),
            len => Ok(len as usize)
        }
    }
}
