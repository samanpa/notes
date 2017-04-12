use std;
use core::time::Time;
use super::iter::*;
use ::error::Error;

pub type LogError = Error;
pub type LogResult<T> = std::result::Result<T,LogError>;

#[repr(C)]
pub struct FileHeader
{
    magic : u32,
    version : u8,
    msg_header_size : u8,
    file_header_size : u8,
}

impl FileHeader
{
    pub fn msg_header_size(&self) -> u8 {
        self.msg_header_size
    }

    pub fn file_header_size(&self) -> u8 {
        self.file_header_size
    }
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

impl MessageHeader
{
    pub fn new(time: Time, seqnum: u64, length: u32, checksum : u32,
               msg_type: u16) -> MessageHeader {
        MessageHeader { time : time,
                        seqnum : seqnum,
                        length: length,
                        checksum : checksum,
                        msg_type : msg_type }
    }

    pub fn time(&self) -> Time {
        self.time.clone()
    }
    
    pub fn seqnum(&self) -> u64 {
        self.seqnum
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn checksum(&self) -> u32 {
        self.checksum
    }

    pub fn msg_type(&self) -> u16 {
       self.msg_type
    }
}

pub struct LogFile
{
    file : std::fs::File,
    seqnum : u64
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
        

pub const MAGIC : u32 = 0xFEEDFACE;
pub const VERSION : u8 = 1;

use std::path::Path;

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
        try!( Self::write_struct(&mut file, &hdr) );
        Ok(LogFile{file: file, seqnum : 0})
    }

    fn validate(file: &mut std::fs::File, path: &Path) -> LogResult<u64> {
        let mut parser = LogFileParser::new(file, path).unwrap();
        let mut iter = parser.parse().unwrap();
        let mut seqnum = 0;

        if iter.header().magic != MAGIC {
            return Err(LogError::from_str("invalid magic"))
        }
        if iter.header().version != VERSION {
            return Err(LogError::from_str("invalid version"))
        }

        for msg in &mut iter {
            match msg {
                Err(err) => return Err(err),
                Ok(_)    => seqnum += 1
            }
        }
        Ok(seqnum)
    }
    
    fn open(path : &Path, perm : Permission) -> std::io::Result<LogFile> {
        use std::io::{Error,ErrorKind};

        let mut file = try!{
            std::fs::OpenOptions::new()
                .write(perm.writeable())
                .read(true)
                .open(path)
        };
        match Self::validate(&mut file, path) {
            Ok(seqnum) => Ok(LogFile{file: file, seqnum : seqnum} ),
            Err(err)   => Err(Error::new(ErrorKind::InvalidData, err))
        }
    }

    pub fn new( path : &str, perm : Permission ) -> LogResult<LogFile> {
        use std::path::Path;

        let path = Path::new(path);
        let func = if path.exists() { Self::open } else { Self::create };
        let file = func(path, perm);
        match file {
            Ok(file) => Ok(file),
            Err(err) => return Err(LogError::new(err.to_string()))
        }
    }

    pub fn write(&mut self, time: Time, msg_type: u16
                 , data: &[u8]) -> LogResult<u64> {
        use std::io::Write;
        let hdr = MessageHeader::new(time, self.seqnum
                                     , data.len() as u32, 0, msg_type);
        let result = Self::write_struct(&mut self.file, &hdr);
        if let Err(err) = result {
            return Err(LogError::new(err.to_string()));
        };
        let result = self.file.write(data);
        match result {
            Err(err) => return Err(LogError::new(err.to_string())),
            _        => {
                self.seqnum += 1;
                Ok(self.seqnum - 1)
            }
        }
    }

    fn write_struct<T>(file: &mut std::fs::File, data: &T) -> std::io::Result<usize> {
        let ptr : *const u8 = data as *const T as *const u8;
        let data = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<T>()) };
        use std::io::Write;
        file.write(data)
    }

}
