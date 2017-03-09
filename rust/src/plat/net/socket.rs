extern crate libc as c;

use std::io::Result;
use std::net::SocketAddrV4;
use super::addr::{into_c_sockaddr,from_c_sockaddr,to_ptr,to_mut_ptr};

pub struct Socket {
    fd : c::c_int
}

impl Socket {
    pub fn new(domain: c::c_int, sock_type: c::c_int, protocol: c::c_int) -> Result<Self> {
        let fd = unsafe{ c::socket(domain, sock_type, protocol) };
        super::to_result(fd).map( |fd| Socket{fd: fd} )
    }

    pub fn from(fd: c::c_int) -> Self {
        Socket{fd: fd}
    }

    pub fn nonblock(&self) -> Result<()> {
        let status_flags = unsafe{ c::fcntl(self.fd, c::F_GETFL)};
        let res = unsafe {
            c::fcntl(self.fd, c::F_SETFL, status_flags | c::O_NONBLOCK)
        };
        super::to_void_result(res)
    }
    
    pub fn fd(&self) -> super::RawFd {
        self.fd
    }

    pub fn connect(&self, addr: &SocketAddrV4) -> Result<()> {
        let addr = into_c_sockaddr(addr);
        let (addrlen, sockaddr) = to_ptr(&addr);
        let ret = unsafe{ c::connect(self.fd(), sockaddr, addrlen) };
        super::to_void_result(ret).or_else( |e| {
            let errno = e.raw_os_error().unwrap();
            match errno as c::c_int {
                c::EINPROGRESS => Ok(()),
                _ => Err(e)
            }
        })
    }

    pub fn bind(&self, addr: &SocketAddrV4) -> Result<()> {
        let addr = into_c_sockaddr(addr);
        let (addrlen,sockaddr) = to_ptr(&addr);
        let ret = unsafe{ c::bind(self.fd, sockaddr, addrlen) };
        super::to_void_result(ret)
    }

    pub fn get_sock_name(&self) -> Result<SocketAddrV4> {
        let inaddr = c::in_addr{ s_addr: 0 };
        let mut addr = c::sockaddr_in{ sin_family: 0,
                                       sin_port: 0,
                                       sin_addr: inaddr,
                                       sin_zero: [0u8; 8]};
        let (mut addrlen, sockaddr) = to_mut_ptr(&mut addr);
        let ret = unsafe{ c::getsockname(self.fd, sockaddr, &mut addrlen as *mut c::socklen_t) };
        super::to_void_result(ret)
            .map( |()| from_c_sockaddr(&addr) )
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe{ c::close(self.fd) };
    }
}
