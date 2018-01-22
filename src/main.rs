extern crate mio;
extern crate bytes;
extern crate unicode_segmentation;

#[macro_use]
extern crate log;

extern crate env_logger;
extern crate reagent;

mod echo;

use std::net::SocketAddr;
use reagent::reactor::Reactor;
use echo::EchoServer;

use reagent::errors::*;

const MAX_CONNECTIONS: usize = 1024;

// The GNU "cfingerd" port, for testing.
const DEFAULT_LISTEN_ADDR: &'static str = "0.0.0.0:2003";
fn serverd_addr() -> SocketAddr {
    DEFAULT_LISTEN_ADDR.parse()
        .expect("Unable to parse address string.")
}


pub fn serve() -> Result<()> {
    let mut reactor = Reactor::new(MAX_CONNECTIONS)?;
    let address = serverd_addr();
    let server = Box::new(EchoServer::new(&address)?);
    reactor.add_agent(server)?;
    reactor.run()
}


pub fn main() {
    env_logger::init().unwrap();
    if let Err(ref err) = serve() {
        println!("error: {}", err);
        for e in err.iter().skip(1) {
            println!("from: {}", e);
        }
        if let Some(backtrace) = err.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }
        ::std::process::exit(1);
    }
}        
