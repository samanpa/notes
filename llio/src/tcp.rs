extern crate libc as c;

use std::io::Result;
use std::net::SocketAddrV4;
use super::Socket;
use super::addr::{from_c_sockaddr,to_mut_ptr};
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

    pub fn from(socket: Socket) -> Result<Self> {
        try!( socket.nonblock());
        Ok(TcpStream{socket: socket})
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

    pub fn has_sock_error(&self) -> Result<()> {
        let mut err : c::c_int = 0;
        try!(self.socket.get_sock_opt(c::SO_ERROR, &mut err));
        match err {
            0 => Ok(()),
            _ => Err(std::io::Error::from_raw_os_error(err))
        }
    }
}


impl std::io::Read for TcpStream {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize> {
        let len = unsafe{ c::read(self.socket.fd()
                                  , buff.as_mut_ptr() as *mut c::c_void
                                  , buff.len()) };
        super::to_result(len)
            .map( |len| len as usize)
    }
}

impl std::io::Write for TcpStream {
    fn write(&mut self, buff: &[u8]) -> Result<usize> {
        let len = unsafe{ c::write(self.socket.fd()
                                   , buff.as_ptr() as *mut c::c_void
                                   , buff.len()) };
        super::to_result(len)
            .map( |len| len as usize)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}


impl TcpListener {
    pub fn new(addr: SocketAddrV4) -> Result<Self> {
        let one: c::c_int = 1;
        let socket = try!(Socket::new(c::AF_INET, c::SOCK_STREAM, 0));
        let _ = try!(socket.nonblock());
        let _ = try!(socket.bind(&addr));
        let _ = try!(socket.set_sock_opt(c::SO_REUSEADDR, &one));
        Ok(TcpListener{ socket: socket, addr: addr })
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

    pub fn fd(&self) -> super::RawFd {
        self.socket.fd()
    }

    pub fn listen(&self, backlog: u32) -> Result<()>{
        let ret = unsafe{ c::listen(self.socket.fd(), backlog as c::c_int) };
        super::to_void_result(ret)
    }
}
