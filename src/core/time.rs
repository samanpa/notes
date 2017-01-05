use std;

pub struct Time {
    pub ns : u64 
}


pub fn now() -> Time {
    let option = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    let duration = option.unwrap();
    let ns = duration.as_secs() * 1_000_000_000 + (duration.subsec_nanos() as u64);
    return Time{ns : ns}
}

