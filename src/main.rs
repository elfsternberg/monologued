extern crate mio;
extern crate bytes;
extern crate unicode_segmentation;

#[macro_use]
extern crate log;
extern crate env_logger;


mod server;
mod tokenqueue;
mod connection;

use std::net::SocketAddr;
use std::str::FromStr;

const SERVER: mio::Token = mio::Token(1);
const MAX_CONNECTIONS: usize = 1024;

// The GNU "cfingerd" port, for testing.
const DEFAULT_LISTEN_ADDR: &'static str = "0.0.0.0:2003";

use server::Server;

fn serverd_addr() -> SocketAddr {
    FromStr::from_str(DEFAULT_LISTEN_ADDR).unwrap()
}

pub fn main() {
    env_logger::init().unwrap();
    let addr = serverd_addr();
    let server = Server::new(addr, SERVER, MAX_CONNECTIONS);
    server.unwrap().run();
}
