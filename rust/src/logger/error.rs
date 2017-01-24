use std;

#[derive(Debug)]
pub struct LogError {
    msg : std::string::String,
}

impl LogError {
    pub fn new(msg : std::string::String) -> LogError {
        LogError{msg: msg }
    }
}

impl std::fmt::Display for LogError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter ) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.msg)
    }
}

impl std::error::Error for LogError {
    fn description(&self) -> &str {
        return &self.msg
    }
}

pub type LogResult<T> = std::result::Result<T,LogError>;
