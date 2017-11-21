extern crate mio;
extern crate nix;
extern crate bytes;

use mio::*;
use mio::tcp::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use mio::util::Slab;
use std::io;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

use std::net::SocketAddr;
use std::str::FromStr;

const SERVER: mio::Token = mio::Token(0);
const MAX_CONNECTIONS: usize = 1024;

// The GNU "cfingerd" port, for testing.
const DEFAULT_LISTEN_ADDR : &'static str= "0.0.0.0:2003";

mod server;
use server::Server;

fn serverd_addr() -> SocketAddr {
    FromStr::from_str(DEFAULT_LISTEN_ADDR).unwrap()
}

pub fn main() {
    let addr = serverd_addr();
    let mut server = Server::new(addr, SERVER);
    server.run().unwrap();
}



