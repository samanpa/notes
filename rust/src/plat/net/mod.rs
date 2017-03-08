pub mod socket;
pub mod epoll;
pub mod tcp;

extern crate libc as c;

use std;
pub use self::socket::Socket as Socket;
pub use self::epoll::Selector as Selector;
pub use self::tcp::TcpStream as TcpStream;

pub enum EventType
{
    Read,
    Write,
    ReadWrite
}

pub type Events = self::epoll::Events;
pub type RawFd = c::c_int;

#[derive(Copy,Clone,Debug)]
pub struct Token(u64);

impl Token {
    pub fn new(id: u64) -> Self {
        Token(id)
    }

    pub fn id(&self) -> u64 {
        self.0
    }
}

impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::cmp::PartialEq for Token {
    fn eq(&self, other: &Token) -> bool {
        return self.0 == other.0
    }
}
impl std::cmp::Eq for Token {}


pub fn to_result(res: c::c_int) -> std::io::Result<()> {
    if res == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
