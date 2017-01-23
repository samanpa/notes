use std;
use core::time::Time;

#[repr(C)]
struct FileHeader
{
    magic : u32,
    version : u8,
    msg_header_size : u8,
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
    file : std::fs::File,
    seqnum : u64
}

#[derive(Debug)]
pub struct LogError
{
    msg : std::string::String,
}

impl std::fmt::Display for LogError
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter ) -> std::result::Result<(), std::fmt::Error>{
        write!(fmt, "{}", self.msg)
    }

}
impl std::error::Error for LogError
{
    fn description(&self) -> &str {
        return &self.msg
    }
}

pub type LogResult<T> = std::result::Result<T,LogError>;

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
const VERSION : u8 = 1;

use std::path::Path;


unsafe fn to_u8<T : Sized>(source: &T) -> &[u8]
{
    let ptr : *const u8 = source as *const T as *const u8;
    std::slice::from_raw_parts(ptr, std::mem::size_of::<T>())
}

unsafe fn from_u8<T : Sized>(source: &[u8]) -> &T
{
    let ptr = source.as_ptr() as *const T;
    &*ptr
}

impl LogFile
{
    fn create(path : &Path, perm : Permission) -> std::io::Result<LogFile> {
        let mut file = try! {
            std::fs::OpenOptions::new()
                .write(perm.writeable())
                .read(true)
                .create(true)
                .open(path)
        };
        let msg_header_size  = std::mem::size_of::<MessageHeader>() as u8;
        let file_header_size = std::mem::size_of::<FileHeader>() as u8;
        let hdr = FileHeader { magic : MAGIC
                               , msg_header_size : msg_header_size
                               , version : VERSION
                               , file_header_size : file_header_size };
        let hdr = unsafe{to_u8(&hdr)};
        use std::io::Write;
        try!{file.write(hdr)};
        Ok(LogFile{file: file, seqnum : 0})
    }

    fn open(path : &Path, perm : Permission) -> std::io::Result<LogFile> {
        use std::os::unix::fs::MetadataExt;
        use std::io::Read;
        use std::io::{Error,Seek,ErrorKind};

        let mut file = try!{
            std::fs::OpenOptions::new()
                .write(perm.writeable())
                .read(true)
                .open(path)
        };
        let mut seqnum = 0;
        let metadata = try!{std::fs::metadata(path)};
        let mut size = metadata.size() as usize;
        let file_header_size = std::mem::size_of::<FileHeader>();
        if size < file_header_size {
            return Err(Error::new(ErrorKind::InvalidData, "file too small"))
        }
        let mut buff : Vec<u8> = vec![0 ; file_header_size];
        try!(file.read(&mut buff));
        let hdr = unsafe { from_u8::<FileHeader>(&buff) };
        if hdr.magic != MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "invalid magic"))
        }
        if hdr.version != VERSION {
            return Err(Error::new(ErrorKind::InvalidData, "invalid version"))
        }
        size -= file_header_size;
        while size > 0 {
            let mut buff : Vec<u8> = vec![0; hdr.msg_header_size as usize];
            println!("{}", size);
            if size < hdr.msg_header_size as usize {
                return Err(Error::new(ErrorKind::InvalidData
                                      , "invalid file size"))
            }
            size -= hdr.msg_header_size as usize;
            try!(file.read(&mut buff));
            let hdr = unsafe { from_u8::<MessageHeader>(&buff) };
            println!("\t --- {}", hdr.length);
            if size < hdr.length as usize {
                return Err(Error::new(ErrorKind::InvalidData
                                      , "invalid message size"))
            }
            size -= hdr.length as usize;
            file.seek(std::io::SeekFrom::Current(0));
            seqnum += 1;
        }
        Ok(LogFile{file: file, seqnum : seqnum} )
    }

    pub fn new( path : &str, perm : Permission ) -> LogResult<LogFile> {
        use std::path::Path;

        let path = Path::new(path);
        let func = if path.exists() { Self::open } else { Self::create };
        let file = func(path, perm);
        match file {
            Ok(file) => Ok(file),
            Err(err) => return Err(LogError{ msg: err.to_string() })
        }
    }

    pub fn write(&mut self, time: Time, msg_type: u16
                 , data: &[u8]) -> LogResult<u64> {
        use std::io::Write;
        let hdr = MessageHeader { time : time,
                                  seqnum : self.seqnum,
                                  length: data.len() as u32,
                                  checksum : 0,
                                  msg_type : msg_type };
        let hdr_bytes = unsafe{to_u8(&hdr)};
        let result = self.file.write(hdr_bytes);
        match result {
            Err(err) => return Err(LogError{ msg: err.to_string() } ),
            _ => ()
        };
        let result = self.file.write(data);
        match result {
            Err(err) => return Err(LogError{ msg: err.to_string() } ),
            _        => {
                self.seqnum += 1;
                Ok(self.seqnum)
            }
        }
    }
}
