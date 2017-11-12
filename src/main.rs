extern crate mio;
extern crate bytes;
extern crate unicode_segmentation;

use mio::*;
use mio::tcp::{TcpListener};
use std::collections::HashMap;
use mio::unix::UnixReady;

const MAX_CONNECTIONS: usize = 1024;
const LISTENER: mio::Token = mio::Token(0);

mod connection;
use connection::Connection;

fn main() {
    let address="0.0.0.0:6567".parse().unwrap();
    let listener = TcpListener::bind(&address).unwrap();
    let mut events = Events::with_capacity(MAX_CONNECTIONS);
    let poll = Poll::new().unwrap();
    let mut connections = HashMap::new();
    let mut count = 1;

    poll.register(&listener, LISTENER, Ready::readable(), PollOpt::edge()).unwrap();
    println!("Listening on {}", address.port());
                  
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                LISTENER => {
                    let (stream, client_addr) = match listener.accept() {
                        Ok((stream, client_addr)) => (stream, client_addr),
                        Err(e) => panic!("Got an error when accepting a connection: {}", e)
                    };
                    // TODO: Replace with a self-incrementing priority queue
                    count += 1;
                    println!("Connection from {}", client_addr);
                    poll.register(&stream,
                                  Token(count),
                                  Ready::readable() |
                                      Ready::writable() |
                                      Ready::from(UnixReady::hup()),
                                  PollOpt::edge())
                        .unwrap();
                    connections.insert(count, Connection::new(stream));
                }


                Token(c) => {
                    println!("Got token {}", c);
                    let done = {
                        let conn = connections.get_mut(&c).unwrap();
                        conn.handle(&event)
                    };

                    if done {
                        let conn = connections.remove(&c).unwrap();
                        poll.deregister(&conn.socket);
                    }
                }
            }
        }
    }
}
 
