use std::error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub enum Error {
    PartialRequest,
    Parse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Parse(ref s) => write!(f, "Invalid method: {}", s),
            Self::PartialRequest => write!(f, "Partial request"),
        }
    }
}
impl error::Error for Error {}
impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::Parse(e.to_string())
    }
}
