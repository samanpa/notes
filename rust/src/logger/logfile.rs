use std;
use core::time::Time;

#[repr(C)]
struct FileHeader
{
    magic : u32,
    msg_header_size : u8,
    version : u8,
    file_header_size : u8,
}

#[repr(C)]
pub struct MessageHeader
{
    time : Time,
    seqnum : u64,
    length : u32,
    checksum : u32,
    msg_type : u16
}

pub struct LogFile
{
    file : std::fs::File
}

pub struct Error
{
    msg : std::string::String,
}

pub enum Permission
{
    Read,
    ReadWrite
}

impl Permission
{
    fn writeable(&self) -> bool {
        match self {
            &Permission::ReadWrite => true,
            _ => false
        }
    }
}
        

const MAGIC : u32 = 0xFEEDFACE;
const VERSION : u8 = 0;

use std::fs::File;
use std::path::Path;


impl LogFile
{
    fn create(path : &Path, perm : Permission) -> std::io::Result<File> {
        let msg_header_size : usize = std::mem::size_of::<MessageHeader>();
        let file_header_size : usize = std::mem::size_of::<FileHeader>();
        let mut file = try! {
            std::fs::OpenOptions::new()
                .write(perm.writeable())
                .create(true)
                .open(path)
        };
        let hdr = FileHeader{ magic : MAGIC
                              , msg_header_size : msg_header_size as u8
                              , version : VERSION
                              , file_header_size : file_header_size as u8};
        let phdr : *const u8 = &hdr as *const FileHeader as *const u8;
        let hdr = unsafe { std::slice::from_raw_parts(phdr, file_header_size)};
        use std::io::Write;
        try!{file.write(hdr)};
        Ok(file)
    }

    fn open(path : &Path, perm : Permission) -> std::io::Result<File> {
        let file = try!{
            std::fs::OpenOptions::new()
                .write(perm.writeable())
                .open(path)
        };
        //TODO: Validate file contents.
        Ok(file)
    }

    pub fn new( path : &str, perm : Permission )
                -> std::result::Result<LogFile,Error> {
        use std::path::Path;

        let path = Path::new(path);
        let func = if path.exists() { Self::open } else { Self::create };
        let file = func(path, perm);
        let file = match file {
            Ok(file) => file,
            Err(err) => return Err(Error{ msg: err.to_string() })
        };
        Ok(LogFile{file : file})
    }

}
