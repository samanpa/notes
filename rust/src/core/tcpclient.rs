extern crate libc;

use core::event::*;

pub enum TcpState {
    Connected,
    Connecting,
    Disconnected,
    NotInitialized,
    Closed
}

pub struct TcpClient {
    fd : Option<libc::c_int>
}
