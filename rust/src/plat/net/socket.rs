extern crate libc as c;

use std::io::{Error,Result};
use std::net::{Ipv4Addr,SocketAddrV4};
use std;

pub struct Socket {
    fd : c::c_int
}


impl Socket {
    pub fn new(domain: c::c_int, sock_type: c::c_int, protocol: c::c_int) -> Result<Self> {
        let fd = unsafe{ c::socket(domain, sock_type, protocol) };
        if fd == -1 {
            return Err(Error::last_os_error());
        }
        Ok(Socket{ fd: fd })
    }


    pub fn nonblock(&mut self) -> Result<()> {
        let status_flags = unsafe{ c::fcntl(self.fd, c::F_GETFL)};
        let res = unsafe {
            c::fcntl(self.fd, c::F_SETFL, status_flags | c::O_NONBLOCK)
        };
        super::to_result(res)
    }
    
    pub fn fd(&self) -> super::RawFd {
        self.fd
    }

    pub fn connect(&self, addr: &SocketAddrV4) -> Result<()> {
        let addr = addr_to_raw(addr);
        let addrlen = std::mem::size_of_val(&addr) as c::socklen_t;
        let sockaddr = (&addr) as *const c::sockaddr_in as *const c::sockaddr;
        let ret = unsafe{ c::connect(self.fd(), sockaddr, addrlen) };
        
        match super::to_result(ret) {
            Ok(_)  => Ok(()),
            Err(e) => {
                let errno = e.raw_os_error().unwrap();
                match errno as c::c_int {
                    c::EINPROGRESS => Ok(()),
                    _ => Err(e)
                }
            }
        }
    }

    pub fn accept(&self) -> Result<(Socket, SocketAddrV4)> {
        let inaddr = c::in_addr{ s_addr: 0 };
        let mut addr = c::sockaddr_in{ sin_family: 0,
                                       sin_port: 0,
                                       sin_addr: inaddr,
                                       sin_zero: [0u8; 8]};

        let mut addrlen = std::mem::size_of_val(&addr) as c::socklen_t;
        let mut sockaddr = (&mut addr) as *mut c::sockaddr_in as *mut c::sockaddr;
        let fd = unsafe{ c::accept(self.fd, sockaddr, &mut addrlen as *mut c::socklen_t) };
        match fd {
            -1 => Err(Error::last_os_error()),
            fd => {
                let addr = raw_to_addr(&addr);
                let sock = Socket{ fd: fd};
                Ok((sock,addr))
            }
        }
    }

    pub fn listen(&self, backlog: u32) -> Result<()>{
        let ret = unsafe{ c::listen(self.fd, backlog as c::c_int) };
        super::to_result(ret)
    }

    pub fn bind(&self, addr: &SocketAddrV4) -> Result<()> {
        let addr = addr_to_raw(addr);
        let addrlen = std::mem::size_of_val(&addr) as c::socklen_t;
        let sockaddr = (&addr) as *const c::sockaddr_in as *const c::sockaddr;
        let ret = unsafe{ c::bind(self.fd, sockaddr, addrlen) };
        super::to_result(ret)
    }

    pub fn get_sock_name(&self) -> Result<SocketAddrV4> {
        let inaddr = c::in_addr{ s_addr: 0 };
        let mut addr = c::sockaddr_in{ sin_family: 0,
                                       sin_port: 0,
                                       sin_addr: inaddr,
                                       sin_zero: [0u8; 8]};

        let mut addrlen = std::mem::size_of_val(&addr) as c::socklen_t;
        let mut sockaddr = (&mut addr) as *mut c::sockaddr_in as *mut c::sockaddr;
        let ret = unsafe{ c::getsockname(self.fd, sockaddr, &mut addrlen as *mut c::socklen_t) };
        match ret {
            -1 => Err(Error::last_os_error()),
            _  => Ok(raw_to_addr(&addr))
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe{ c::close(self.fd) };
    }
}

//Find better name
pub fn addr_to_raw(addr: &SocketAddrV4) -> c::sockaddr_in {
    let ip = addr.ip();
    let octet = ip.octets();
    let inaddr = c::in_addr{ s_addr: (((octet[0] as u32) << 24) |
                                      ((octet[1] as u32) << 16) |
                                      ((octet[2] as u32) <<  8) |
                                      (octet[3] as u32)).to_be() };
    let addr = c::sockaddr_in{ sin_family: c::AF_INET as u16,
                               sin_port: addr.port().to_be(),
                               sin_addr: inaddr,
                               sin_zero: [0u8; 8]};
    addr
}

fn raw_to_addr(addr: &c::sockaddr_in) -> SocketAddrV4 {
    let port  = addr.sin_port as u16;
    let bits  = addr.sin_addr.s_addr.to_be();
    let octet = [(bits >> 24) as u8, (bits >> 16) as u8, (bits >> 8) as u8, bits as u8];
    let ip = Ipv4Addr::new(octet[0], octet[1], octet[2], octet[3]);
    SocketAddrV4::new(ip, port.to_be())
}
