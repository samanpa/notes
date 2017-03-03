extern crate libc as c;

use core;
use core::error::{Error,Result};
use std::net::SocketAddrV4;
use std;

pub struct Socket {
    fd : c::c_int
}


impl Socket {
    pub fn new(domain: c::c_int, sock_type: c::c_int, protocol: c::c_int) -> Result<Self> {
        let fd = unsafe{ c::socket(domain, sock_type, protocol) };
        if fd == -1 {
            return Err(Error::from_str("Could not create string"));
        }
        Ok(Socket{ fd: fd })
    }


    pub fn nonblock(&mut self) -> Result<()> {
        let status_flags = unsafe{ c::fcntl(self.fd, c::F_GETFL)};
        let res = unsafe {
            c::fcntl(self.fd, c::F_SETFL, status_flags | c::O_NONBLOCK)
        };
        Ok(())
    }
    
    pub fn fd(&self) -> c::c_int {
        self.fd
    }

    pub fn connect(&self, addr: &SocketAddrV4) -> Result<()> {
        let addr = addr2raw(addr);
        let addrlen = std::mem::size_of_val(&addr) as c::socklen_t;
        let sockaddr = (&addr) as *const c::sockaddr_in as *const c::sockaddr;
        let ret = unsafe{ c::connect(self.fd(), sockaddr, addrlen) };

        match core::to_result(ret) {
            Ok(_)  => Ok(()),
            Err(e) => {
                let errno = e.raw_os_error().unwrap();
                match errno as c::c_int {
                    c::EINPROGRESS => Ok(()),
                    _ => Err(Error::from_str(std::error::Error::description(&e)))
                }
            }
        }

    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe{ c::close(self.fd) };
    }
}

//Find better name
pub fn addr2raw(addr: &SocketAddrV4) -> c::sockaddr_in {
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
