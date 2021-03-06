use std::time::Duration;
use std;

#[repr(C)]
#[derive(Eq)]
pub struct Time {
    ns : u64 
}

impl Clone for Time {
    fn clone(&self) -> Time {
        Time::new(self.ns)
    }
}

impl Copy for Time {}

impl std::cmp::PartialEq for Time {
    fn eq(&self, other: &Time) -> bool {
        self.ns == other.ns
    }
}
impl std::cmp::PartialOrd for Time {
    fn partial_cmp(&self, other: &Time) -> Option<std::cmp::Ordering> {
        self.ns.partial_cmp(&other.ns)
    }
}
impl std::cmp::Ord for Time {
    fn cmp(&self, other: &Time) -> std::cmp::Ordering {
        self.ns.cmp(&other.ns)
    }
}

impl std::ops::Add<Duration> for Time {
    type Output = Time;
    fn add(self, duration: Duration) -> Time {
        let dur_ns = duration.as_secs() * 1000_000_000u64 + duration.subsec_nanos() as u64;
        return Time::new(self.ns + dur_ns)
    }
}

impl std::ops::Sub<Time> for Time {
    type Output = Duration;
    fn sub(self, other: Time) -> Duration {
        let res = self.ns - other.ns;
        return Duration::new(res / 1000_000_000u64
                             , (res % 1000_000_000u64) as u32);
    }
}

impl Time {
    fn new(ns : u64) -> Time {
        Time{ ns: ns }
    }

    pub fn min() -> Time {
        Time::new(0)
    }

    pub fn now() -> Time {
        now()
    }
}

#[cfg(not(target_os = "linux"))]
fn now() -> Time {
    let option = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    let duration = option.unwrap();
    let ns = duration.as_secs() * 1_000_000_000 + (duration.subsec_nanos() as u64);
    Time::new(ns)
}


#[cfg(target_os = "linux")]
fn now() -> Time {
    extern crate libc;
    let mut time = libc::timespec{tv_sec: 0, tv_nsec : 0 };
    let mut ns = 0;
    unsafe {
        let res = libc::clock_gettime(libc::CLOCK_REALTIME, &mut time);
        if res == 0 {
            ns = time.tv_sec as u64 * 1_000_000_000 + time.tv_nsec as u64;
        }
    };
    Time::new(ns)
}
