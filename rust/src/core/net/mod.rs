pub mod tcp;
pub mod socket;

extern crate libc as c;
use std;


//use names for futures.rs for now
enum Async<T> {
    Ready(T),
    NotReady,
}

type Poll<T, E> = Result<Async<T>, E>;

pub fn to_result(res: c::c_int) -> std::io::Result<()> {
    if res == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
