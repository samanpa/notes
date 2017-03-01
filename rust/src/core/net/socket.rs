extern crate libc as c;

use core::error::{Error,Result};

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
            c::fcntl(self.fd, c::F_GETFL, status_flags & c::O_NONBLOCK)
        };
        Ok(())
    }
    
    pub fn fd(&self) -> c::c_int {
        self.fd
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe{ c::close(self.fd) };
    }
}
