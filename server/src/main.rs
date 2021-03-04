mod server;
mod error;

use error::Error;
use server::Server;
use std::error::Error as StdError;

fn main() -> Result<(), Box<dyn StdError>> {
  let server = Server::new("localhost:3000");
  server.run()?;
  Ok(())
}
