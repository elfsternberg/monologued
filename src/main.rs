extern crate mio;
    
use mio::*;
use mio::tcp::{TcpListener, TcpStream};

use std::io::{Read, Write};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const MAX_CONNECTIONS: usize = 1024;
const MAX_LINE: usize = 128;
const LISTENER: mio::Token = mio::Token(0);


struct Connection {
    client_addr: SocketAddr,
    handle: TcpStream,
}

impl Connection {
    fn new(handle: TcpStream, address: SocketAddr) -> Self {
        Connection {
            client_addr: address,
            handle: handle
        }
    }
}

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
                    let (stream, client_addr) = match listener.accept() {
                        Ok((stream, client_addr)) => (stream, client_addr),
                        Err(e) => panic!("Got an error when accepting a connection: {}", e)
                    };
                    count += 1;
                    println!("Connection from {}", client_addr);
                    poll.register(&stream, Token(count), Ready::readable() | Ready::writable(), PollOpt::edge())
                        .unwrap();
                    let conn = Connection::new(stream, client_addr);
                    connections.insert(count, conn);
                }
                Token(c) => {
                    println!("Got token {}", c);
                    {
                        let conn = connections.get_mut(&c).unwrap();
                        if event.readiness().is_writable() {
                            let message = format!("Hello, {}\r\n", conn.client_addr).into_bytes();
                            match conn.handle.write(&message) {
                                Ok(n) => println!("Wrote to client {}", conn.client_addr),
                                Err(e) => println!("Error when writing to connection {}", e)
                            }
                        }
                    }
                    
                    let conn = connections.remove(&c).unwrap();
                    poll.deregister(&conn.handle);
                }
            }
        }
    }
}
 
