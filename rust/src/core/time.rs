#[repr(C)]
pub struct Time {
    pub ns : u64 
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
    use core::linux;
    return Time{ ns : linux::now() }
}
