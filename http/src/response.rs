use crate::Headers;
use crate::Status;
use crate::Version;

use std::io::{Result, Write};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Response {
  version: Version,
  status: Status,
  headers: Headers,
  body: Option<String>,
}
impl Default for Response {
  fn default() -> Self {
    Self {
      version: Version::V1_1,
      status: Status::Ok,
      headers: Headers::new(),
      body: None,
    }
  }
}
impl fmt::Display for Response {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {}\r\n{}\r\n\r\n{}",
      self.version.to_string(), self.status.to_string(),
      self.headers.to_string(), self.body())
  }
}

impl Response {
  pub fn new(status: Status,
             headers: Headers,
             body: Option<String>) -> Self {
    let mut headers = headers;
    if !headers.contains_key(Headers::CONTENT_TYPE) {
      headers.insert(Headers::CONTENT_TYPE, "text/html");
    }
    if let Some(ref b) = body {
      if !headers.contains_key(Headers::CONTENT_LENGTH) {
        headers.insert(Headers::CONTENT_LENGTH, &b.len().to_string())
      }
    }

    Self {
      version: Version::V1_1,
      status,
      headers,
      body,
    }
  }

  pub fn send(&self, s: &mut impl Write) -> Result<()> {
    write!(s, "{}", self.to_string())
  }

  pub fn body(&self) -> &str {
    match &self.body {
      Some(b) => b.as_str(),
      None => "",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_default() {
    let r = Response::default();
    let actual = Response {
      version: Version::V1_1,
      status: Status::Ok,
      headers: Headers::new(),
      body: None,
    };
    assert_eq!(actual, r);
  }

  #[test]
  fn create_200() {
    let mut h = Headers::new();
    h.insert("server", "archive");

    let r = Response::new(
      Status::Ok,
      h,
      Some("This is the body text, it has stuff in it. Length 52".into()));

    let actual = Response {
      version: Version::V1_1,
      status: Status::Ok,
      headers: {
        let mut h = Headers::new();
        h.insert("server", "archive");
        h.insert("content-type", "text/html");
        h.insert("content-length", "52");
        h
      },
      body: Some("This is the body text, it has stuff in it. Length 52".into()),
    };
    assert_eq!(actual, r);
  }

  #[test]
  fn create_200_with_content_length_and_type() {
    let mut h = Headers::new();
    h.insert("server", "archive");
    h.insert(Headers::CONTENT_LENGTH, "200");
    h.insert(Headers::CONTENT_TYPE, "application/xml");

    let r = Response::new(
      Status::Ok,
      h,
      Some("This is the body text, it has stuff in it. Length 52".into()));

    let actual = Response {
      version: Version::V1_1,
      status: Status::Ok,
      headers: {
        let mut h = Headers::new();
        h.insert("server", "archive");
        h.insert("content-type", "application/xml");
        h.insert("content-length", "200");
        h
      },
      body: Some("This is the body text, it has stuff in it. Length 52".into()),
    };
    assert_eq!(actual, r);
  }
}
