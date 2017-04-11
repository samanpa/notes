use std;

pub struct Error {
}

pub type Result<T> = std::result::Result<T, std::io::Error>;

impl Error {
    pub fn new(msg : std::string::String) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, msg)
    }

    pub fn from_str(msg : &str) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, msg)
    }
}
