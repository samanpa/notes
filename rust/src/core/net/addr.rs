extern crate libc as c;

use std;
use std::net::{SocketAddrV4};
use core::net::socket::Socket;

pub fn connect(socket:&Socket, addr: &SocketAddrV4) -> c::c_int
{
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
    let addrlen = std::mem::size_of::<c::sockaddr_in>() as c::socklen_t;
    let sockaddr = (&addr) as *const c::sockaddr_in as *const c::sockaddr;
    unsafe{ c::connect(socket.fd(), sockaddr, addrlen) }
}
