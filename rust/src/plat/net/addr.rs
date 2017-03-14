extern crate libc as c;

use std::net::{Ipv4Addr,SocketAddrV4};
use std;

pub fn into_c_sockaddr(addr: &SocketAddrV4) -> c::sockaddr_in {
    let ip = addr.ip();
    let octet = ip.octets();
    let inaddr = c::in_addr{ s_addr: (((octet[0] as u32) << 24) |
                                      ((octet[1] as u32) << 16) |
                                      ((octet[2] as u32) <<  8) |
                                      (octet[3] as u32)).to_be() };
    let addr = c::sockaddr_in{ sin_family: c::AF_INET as u16,
                               sin_port: u16::from_be(addr.port()),
                               sin_addr: inaddr,
                               sin_zero: [0u8; 8]};
    addr
}


pub fn from_c_sockaddr(addr: &c::sockaddr_in) -> SocketAddrV4 {
    let port  = addr.sin_port as u16;
    let bits  = addr.sin_addr.s_addr.to_be();
    let octet = [(bits >> 24) as u8, (bits >> 16) as u8, (bits >> 8) as u8, bits as u8];
    let ip = Ipv4Addr::new(octet[0], octet[1], octet[2], octet[3]);
    SocketAddrV4::new(ip, port.to_be())
}

pub fn to_ptr(addr: &c::sockaddr_in) -> (c::socklen_t, *const c::sockaddr) {
    let addrlen = std::mem::size_of_val(addr) as c::socklen_t;
    let sockaddr = addr as *const c::sockaddr_in as *const c::sockaddr;
    (addrlen,sockaddr)
}

pub fn to_mut_ptr(addr: &mut c::sockaddr_in) -> (c::socklen_t, *mut c::sockaddr) {
    let addrlen = std::mem::size_of_val(addr) as c::socklen_t;
    let sockaddr = addr as *mut c::sockaddr_in as *mut c::sockaddr;
    (addrlen,sockaddr)
}
