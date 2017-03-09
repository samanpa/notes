extern crate libc as c;

use std::io::{Error,ErrorKind,Result};
use std::net::{Ipv4Addr,SocketAddrV4};
use super::Socket;
use super::addr::{into_c_sockaddr,from_c_sockaddr,to_ptr,to_mut_ptr};
use std;

pub struct TcpStream {
    socket : Socket,
}

pub struct TcpListener {
    socket : Socket,
    addr   : SocketAddrV4,
}
    

impl TcpStream {
    pub fn new() -> Result<Self> {
        Socket::new(c::AF_INET, c::SOCK_STREAM, 0)
            .map( |socket| TcpStream{ socket: socket } )
    }

    pub fn nonblock(&self) -> Result<()> {
        self.socket.nonblock()
    }

    pub fn connect(&self, addr: &SocketAddrV4) -> Result<()> {
        self.socket.connect(addr)
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


impl TcpListener {
    pub fn new(addr: SocketAddrV4) -> Result<Self> {
        Socket::new(c::AF_INET, c::SOCK_STREAM, 0)
            .map( |socket| TcpListener{ socket: socket, addr: addr } )
    }

    pub fn accept(&self) -> Result<(Socket, SocketAddrV4)> {
        let inaddr = c::in_addr{ s_addr: 0 };
        let mut addr = c::sockaddr_in{ sin_family: 0,
                                       sin_port: 0,
                                       sin_addr: inaddr,
                                       sin_zero: [0u8; 8]};
        let (mut addrlen, sockaddr) = to_mut_ptr(&mut addr);
        let fd = unsafe{ c::accept(self.socket.fd(), sockaddr, &mut addrlen as *mut c::socklen_t) };
        super::to_result(fd).map( |fd| {
            let addr = from_c_sockaddr(&addr);
            let sock = Socket::from(fd);
            (sock,addr)
        })
    }

    pub fn listen(&self, backlog: u32) -> Result<()>{
        let ret = unsafe{ c::listen(self.socket.fd(), backlog as c::c_int) };
        super::to_void_result(ret)
    }


}
