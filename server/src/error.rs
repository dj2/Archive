use http::Error as HttpError;
use std::io::Error as IoError;
use std::string::FromUtf8Error as Utf8Error;
use std::fmt;
use std::error;

#[derive(Debug)]
pub enum Error {
  HttpError(HttpError),
  IoError(IoError),
  Utf8Error(Utf8Error),
}
impl From<IoError> for Error {
  fn from(e: IoError) -> Self {
      Self::IoError(e)
  }
}
impl From<Utf8Error> for Error {
  fn from(e: Utf8Error) -> Self {
    Self::Utf8Error(e)
  }
}
impl From<HttpError> for Error {
  fn from(e: HttpError) -> Self {
    Self::HttpError(e)
  }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::HttpError(e) => write!(f, "{}", e),
            Self::IoError(e) => write!(f, "{}", e),
            Self::Utf8Error(e) => write!(f, "{}", e),
        }
    }
}
impl error::Error for Error {}
