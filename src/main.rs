extern crate mio;
extern crate bytes;
extern crate unicode_segmentation;
extern crate slab;

#[macro_use]
extern crate log;

mod server;
mod tokenqueue;
mod connection;

use mio::*;
use mio::tcp::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

use std::net::SocketAddr;
use std::str::FromStr;

const SERVER: mio::Token = mio::Token(0);
const MAX_CONNECTIONS: usize = 1024;

// The GNU "cfingerd" port, for testing.
const DEFAULT_LISTEN_ADDR: &'static str = "0.0.0.0:2003";

use server::Server;

fn serverd_addr() -> SocketAddr {
    FromStr::from_str(DEFAULT_LISTEN_ADDR).unwrap()
}

pub fn main() {
    let addr = serverd_addr();
    let mut server = Server::new(addr, SERVER, MAX_CONNECTIONS);
    server.unwrap().run();
}
