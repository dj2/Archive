use crate::Error;

use http::Request;
use std::convert::TryInto;
use std::net::TcpListener;
use std::io::Read;

pub struct Server<'a> {
  addr: &'a str,
}
impl<'a> Server<'a> {
  pub fn new(addr: &'a str) -> Self {
    Self { addr }
  }

  pub fn run(&self) -> Result<(), Error> {
    let conn = TcpListener::bind(self.addr)?;
    println!("Running on {}", self.addr, conn);

    for stream in conn.incoming() {
      let mut stream = stream?;

      let mut buf = [0; 512];
      stream.read(&mut buf)?;

      let req: Request = String::from_utf8_lossy(buf.to_vec())?.try_into()?;
      println!("{:?}", req);
    }
    Ok(())
  }
}
