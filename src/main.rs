extern crate mio;
extern crate bytes;
extern crate toml;
extern crate clap;
extern crate rfc1288;
    
#[macro_use] 
extern crate log;

use mio::*;
use mio::tcp::{TcpListener};

use std::io::{Read, Write};
use std::collections::HashMap;
use bytes::{Buf};

const MAX_CONNECTIONS: usize = 1024;
const MAX_LINE: usize = 128;
const LISTENER: mio::Token = mio::Token(0);

mod transaction;
use transaction::Transaction;

fn main() {
    let address="0.0.0.0:6567".parse().unwrap();
    let listener = TcpListener::bind(&address).unwrap();
    let mut events = Events::with_capacity(MAX_CONNECTIONS);
    let poll = Poll::new().unwrap();
    let mut connections = HashMap::new();
    let mut count = 1;

    poll.register(&listener, LISTENER, Ready::readable(), PollOpt::edge()).unwrap();

    // There are two possible kinds of events that the Poll endpoint
    // cares about: Listen() events, which means that a new
    // transaction has come in, and Token() events, which means that a
    // the kernel has something capable for a given Token.
    
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
                            txn.register_to_read(&mut poll);
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
                            conn.read(&mut poll);
                        }

                        if event.readiness().is_writable() {
                            match conn.write(&mut poll) {
                                Ok(n) => println!("Wrote to client {}", conn.client_addr),
                                Err(e) => println!("Error when writing to connection {}", e)
                            }
                            
                        }
                    }
                }
            }
        }
    }
}
 
