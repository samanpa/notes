pub mod addr;
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


trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}
impl IsMinusOne for i32 {
    fn is_minus_one(&self) -> bool { *self == -1 }
}
impl IsMinusOne for isize {
    fn is_minus_one(&self) -> bool { *self == -1 }
}
fn cvt<T: IsMinusOne>(t: T) -> std::io::Result<T> {
    if t.is_minus_one() {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

fn to_void_result(res: c::c_int) -> std::io::Result<()> {
    cvt(res)
        .map( |_| {} )
}

fn to_result<T: IsMinusOne>(res: T) -> std::io::Result<T> {
    cvt(res)
}
