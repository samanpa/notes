
pub enum Protection
{
    None,
    Exec,
    Read,
    Write,
    ReadWrite
}

impl Into<i32> for Protection
{
    //from include/asm-ia64/mman.h
    fn into(self) -> i32 {
        match self {
            Protection::None => 0,
            Protection::Exec   => 4,
            Protection::Read   => 1,
            Protection::Write  => 2,
            Protection::ReadWrite => 6
        }
    }
}

struct MmapRegion
{
    data : *const u8,
    size : u64
}

use std::fs::File;
pub struct MmapFile
{
    data : MmapRegion,
    file : File,
}

pub fn open<P: AsRef<Path>>(path: P) -> MmapFile
{

    let mut f = try!(File::open(path));
}
