extern crate mio;
extern crate bytes;
extern crate unicode_segmentation;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate error_chain;

mod reagent;
mod reactor;
mod echo;
mod tokenqueue;
mod errors;

use std::net::SocketAddr;
use std::str::FromStr;
use reactor::Reactor;
use echo::EchoServer;
pub use errors::*;

const MAX_CONNECTIONS: usize = 1024;

// The GNU "cfingerd" port, for testing.
const DEFAULT_LISTEN_ADDR: &'static str = "0.0.0.0:2003";
fn serverd_addr() -> Result<SocketAddr> {
    Ok(try!(FromStr::from_str(DEFAULT_LISTEN_ADDR)))
}


pub fn serve() -> Result<()> {
    let reactor = try!(Reactor::new(MAX_CONNECTIONS));
    let address = try!(serverd_addr());
    let server = try!(EchoServer::new(&address));
    try!(reactor.add_agent(server));
    Ok(try!(reactor.run()))
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
