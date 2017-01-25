#[repr(C)]
pub struct Time {
    ns : u64 
}

impl Clone for Time {
    fn clone(&self) -> Time {
        Time{ns : self.ns}
    }
}

use std;
use std::cmp::Ordering;
impl std::cmp::PartialOrd for Time {
    fn partial_cmp(&self, other: &Time) -> Option<Ordering> {
        Some(self.ns.partial_cmp(other.ns))
    }
}

#[cfg(not(target_os = "linux"))]
pub fn now() -> Time {
    use std;
    let option = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    let duration = option.unwrap();
    let ns = duration.as_secs() * 1_000_000_000 + (duration.subsec_nanos() as u64);
    return Time{ns : ns}
}


#[cfg(target_os = "linux")]
pub fn now() -> Time {
    extern crate libc;
    let mut time = libc::timespec{tv_sec: 0, tv_nsec : 0 };
    let mut ns = 0;
    unsafe {
        let res = libc::clock_gettime(libc::CLOCK_REALTIME, &mut time);
        if res == 0 {
            ns = time.tv_sec as u64 * 1_000_000_000 + time.tv_nsec as u64;
        }
    }
    return Time{ ns : ns }
}
