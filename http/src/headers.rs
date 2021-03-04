use std::collections::HashMap;
use std::fmt;

/// HTTP headers.
#[derive(Clone, Debug, PartialEq)]
pub struct Headers {
  data: HashMap<String, String>,
}
impl Headers {
  pub const ACCEPT: &'static str = "accept";
  pub const CONTENT_LENGTH: &'static str = "content-length";
  pub const CONTENT_TYPE: &'static str = "content-type";
  pub const DATE: &'static str = "date";
  pub const HOST: &'static str = "host";
  pub const SERVER: &'static str = "server";
  pub const USER_AGENT: &'static str = "user-agent";

  pub fn new() -> Self {
    Self {
      data: HashMap::new()
    }
  }

  pub fn contains_key(&self, a: &str) -> bool {
    self.data.contains_key(a)
  }
  pub fn insert(&mut self, a: &str, b: &str) {
    self.data.insert(a.to_lowercase(), b.into());
  }

  pub fn get(&self, a: &str) -> Option<&String> {
    return self.data.get(&a.to_lowercase());
  }
}
impl fmt::Display for Headers {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for (k, v) in &self.data {
      write!(f, "{}: {}\n", k, v)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_and_get() {
    let mut h = Headers::new();
    assert_eq!(None, h.get(Headers::USER_AGENT));

    h.insert("A Header", "data");
    assert_eq!(Some(&"data".to_string()), h.get("a Header"));
    assert_eq!(None, h.get("Other"));
  }

  #[test]
  fn to_string() {
    let mut h = Headers::new();
    h.insert(Headers::HOST, "localhost");
    h.insert(Headers::ACCEPT, "*/*");
    h.insert(Headers::CONTENT_TYPE, "application/xml");

    // Convert to string, then resort the headers into alphabetical order
    // to make comparison easier.
    let s = h.to_string();
    let mut s: Vec<&str> = s.lines().collect::<Vec<&str>>();
    s.sort();
    let data = s.join("\n");

    assert_eq!("accept: */*
content-type: application/xml
host: localhost", data);
  }
}
