extern crate mio;
extern crate bytes;
extern crate toml;
extern crate clap
    
use mio::*;
use mio::tcp::{TcpListener, TcpStream};

use std::io::{Read, Write, File};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use bytes::{Bytes, BytesMut, BufMut, Buf};
use std::fmt::Display;

const MAX_CONNECTIONS: usize = 1024;
const MAX_LINE: usize = 128;
const LISTENER: mio::Token = mio::Token(0);



fn main() {
    let address="0.0.0.0:6567".parse().unwrap();
    let listener = TcpListener::bind(&address).unwrap();
    let mut events = Events::with_capacity(MAX_CONNECTIONS);
    let poll = Poll::new().unwrap();
    let mut connections = HashMap::new();
    let mut count = 1;

    poll.register(&listener, LISTENER, Ready::readable(), PollOpt::edge()).unwrap();
                  
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                LISTENER => {
                    match listener.accept() {
                        Ok((socket, client_addr)) => {
                            count += 1;
                            let txn = Transaction::new(socket, client_addr, Token(count));
                            connections.insert(count, txn);
                            txn.register_to_read(&poll);
                            println!("Connection from {}", client_addr);
                        }
                        Err(e) => panic!("Got an error when accepting a connection: {}", e)
                    };
                }
                Token(c) => {
                    println!("Got token {}", c);

                    {
                        let conn = connections.get_mut(&c).unwrap();
                        if event.readiness().is_readable() {
                            conn.read(&poll);
                        }

                        if event.readiness().is_writable() {
                            match conn.write(&poll) {
                                Ok(n) => println!("Wrote to client {}", conn.client_addr),
                                Err(e) => println!("Error when writing to connection {}", e)
                            }
                            
                        }
                    }
                    if event.readiness().is_writable() {
                        let conn = connections.remove(&c).unwrap();
                        poll.deregister(&conn.handle);
                    }
                }
            }
        }
    }
}
 
