#[allow(non_camel_case_types)]
type time_t = u32;
#[allow(non_camel_case_types)]
type long = u64;
#[allow(non_camel_case_types)]
type clock_id = u32;

#[repr(C)]
struct timespec {
    tv_sec : time_t,
    tv_nsec : long
}


pub fn now() -> u64
{
    let mut time = timespec{tv_sec: 0, tv_nsec : 0 };
    unsafe {
        let res = clock_gettime(0, &mut time);
        if res == 0 {
            time.tv_sec as u64 * 1_000_000_000 + time.tv_nsec
        }
        else {
            0
        }
    }

}

#[link(name="rt")]
extern {
    fn clock_gettime( clk_id : clock_id, res : *mut timespec) -> i32;
}


