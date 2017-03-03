use std;

#[derive(Debug)]
pub struct Error {
    msg : std::string::String,
}

pub type Result<T> = std::result::Result<T, Error>;


impl Error {
    pub fn new(msg : std::string::String) -> Error {
        Error{msg: msg}
    }
    pub fn from_str(msg : &str) -> Error {
        Error{msg: msg.to_string()}
    }
    pub fn from_err<E: std::error::Error>(e: E) -> Error {
        Error::from_str(std::error::Error::description(&e))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter ) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.msg)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        return &self.msg
    }
}
