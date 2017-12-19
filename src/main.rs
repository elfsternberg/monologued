extern crate mio;
extern crate bytes;
extern crate unicode_segmentation;

#[macro_use]
extern crate log;
extern crate env_logger;

mod reagent;
mod reactor;
mod echo;
mod tokenqueue;

use std::net::SocketAddr;
use std::str::FromStr;
use reactor::Reactor;
use echo::EchoServer;

const MAX_CONNECTIONS: usize = 1024;

// The GNU "cfingerd" port, for testing.
const DEFAULT_LISTEN_ADDR: &'static str = "0.0.0.0:2003";
fn serverd_addr() -> SocketAddr {
    FromStr::from_str(DEFAULT_LISTEN_ADDR).unwrap()
}

pub fn main() {
    env_logger::init().unwrap();
    let reactor = Reactor::new(MAX_CONNECTIONS);
    let server = EchoServer::new(serverd_addr());
    reactor.add_agent(server);
    reactor.run()
}
