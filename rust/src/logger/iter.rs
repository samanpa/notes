use std;
use logger::logfile::*;

pub struct Message {
    pub hdr : MessageHeader,
    pub data : Vec<u8>
}


pub struct MessageIterator<'a> {
    file : &'a std::fs::File,
    header_size : usize,
    remaining_size : i64
}

unsafe fn from_u8<T : Sized>(source: &[u8]) -> &T
{
    let ptr = source.as_ptr() as *const T;
    &*ptr
}


fn error<T>(msg: &str) -> Option<LogResult<T>> {
    Some(Err(LogError::new(String::from(msg))))
}

impl <'a> Iterator for MessageIterator<'a> {
    type Item = LogResult<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        use std::io::Read;

        if self.remaining_size <= 0 {
            return None;
        }

        let mut buff : Vec<u8> = vec![0; self.header_size];
        if (self.remaining_size as usize) < self.header_size {
            self.remaining_size = 0;
            return error("Not enough space for header");
        }
        
        self.remaining_size -= self.header_size as i64;
        if let Err(err) = self.file.read(&mut buff) {
            self.remaining_size = 0;
            return error(&err.to_string());
        }
        
        let hdr = unsafe { from_u8::<MessageHeader>(&buff) };
        if self.remaining_size < hdr.length() as i64 {
            self.remaining_size = 0;
            return error("invalid message size");
        }
        
        self.remaining_size -= hdr.length() as i64;
        let mut buff : Vec<u8> = vec![0; hdr.length() as usize];
        if let Err(err) = self.file.read(&mut buff) {
            return error(&(err.to_string() + " could not read datA"));
        }

        Some(Ok(Message{hdr: MessageHeader::new(hdr.time(), hdr.seqnum(), hdr.length(), 0, hdr.msg_type()), data: buff}))
    }
}



pub struct LogFileParser<'a> {
    file : &'a std::fs::File,
    size : i64,
    buff : Vec<u8>
}

pub struct LogFileIterator<'a>
{
    file: &'a std::fs::File,
    header: &'a FileHeader,
    size : i64
}

impl<'a> LogFileParser<'a> {
    pub fn new(file: &'a std::fs::File, path: &'a std::path::Path)
               -> LogResult<Self> {
        use std::os::unix::fs::MetadataExt;

        let metadata = std::fs::metadata(path);
        match metadata {
            Err(err) => error(&err.to_string()).unwrap(),
            Ok(metadata) => {
                let size = metadata.size() as i64;
                Ok(LogFileParser{file: file, size: size, buff: Vec::new() })
            }
        }
    }
    
    pub fn parse(&'a mut self) -> LogResult<LogFileIterator<'a>> {
        use std::io::Read;
        
        let file_header_size = std::mem::size_of::<FileHeader>();
        if (self.size as usize) < file_header_size {
            return error("file does not have header too small").unwrap();
        }

        self.buff = vec![0; file_header_size];
        if let Err(_) = self.file.read_exact(&mut self.buff) {
            return error("Could not read header").unwrap();
        }
        
        let hdr = unsafe { from_u8::<FileHeader>(&self.buff) };

        let size = self.size - (hdr.file_header_size() as i64);

        //FIXME: skip hdr.file_header_size - file_header_size
        Ok(LogFileIterator{ file: self.file, header : hdr, size: size})
    }
}


impl <'a> LogFileIterator<'a>
{
    pub fn header(&self) -> &'a FileHeader {
        return self.header;
    }
}

impl <'a> IntoIterator for &'a mut LogFileIterator<'a>
{
    type Item = LogResult<Message>;
    type IntoIter = MessageIterator<'a>;

    fn into_iter(self) -> MessageIterator<'a> {
        MessageIterator{ file : self.file,
                         header_size : self.header.msg_header_size() as usize,
                         remaining_size : self.size }
    }
}

