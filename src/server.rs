use tokenqueue::NextFree;
use connection::{Connection, ConnectionState};
use std::collections::HashMap;

use mio::*;
use std;
use mio::tcp::TcpListener;
use std::net::SocketAddr;
use std::str::FromStr;
use std::io;
use std::fmt;
use std::convert::From;
use std::fmt::Display;

pub struct Server {
    socket: TcpListener,
    token: Token,
    connections: HashMap<Token, Connection>,
    events: Events,
    poll: Poll,
    tokens: NextFree,
}


impl Server {
    pub fn new(addr: SocketAddr, token: Token, max_connections: usize) -> io::Result<Server> {
        let socket = try!(TcpListener::bind(&addr));
        let poll = try!(Poll::new());
        try!(poll.register(&socket, token, Ready::readable(), PollOpt::edge()));

        info!("Starting server on address {:?}", addr);
        Ok(Server {
            socket: socket,
            token: token,
            connections: HashMap::new(),
            events: Events::with_capacity(max_connections),
            tokens: NextFree::new(From::from(token)),
            poll: poll,
        })
    }

    pub fn run(&mut self) {
        let listener = self.token;
        loop {
            match self.poll.poll(&mut self.events, None) {
                Ok(event_count) => {
                    for event in self.events.iter() {
                        if event.token() == self.token {
                            self.accept();
                            continue;
                        } else {
                            self.message(event.token(), event.kind())
                        }
                    }
                }
                Err(e) => {
                    error!("Error while polling: {:?}", e);
                }
            }
        }
    }

    fn accept(&mut self) {
        match self.socket.accept() {
            Ok((socket, client_addr)) => {
                info!("Connection from {:?}", client_addr);
                let next_token = self.tokens.pop();
                let connection = Connection::new(socket, Token(next_token));
                match self.poll.register(&connection.socket,
                                         connection.token,
                                         connection.ready,
                                         PollOpt::edge()) {
                    Ok(_) => {
                        self.connections.insert(Token(next_token), connection);
                    }
                    Err(e) => {
                        error!("Error while adding connection {:?}: {:?}", next_token, e);
                    }
                }
            }
            Err(e) => {
                error!("Error while accepting connection: {:?}", e);
            }
        }
    }

    fn message(&mut self, c: Token, kind: Ready) {
        info!("Got token {:?}", c);
        let status = match self.connections.get_mut(&c) {
            Some(conn) => conn.message(&kind),
            None => {
                error!("Failed to look up connection {:?}", c);
                return;
            }
        };

        match status {
            ConnectionState::Closed => {
                match self.connections.remove(&c) {
                    Some(conn) => {
                        match self.poll.deregister(&conn.socket) {
                            Ok(_) => self.tokens.push(std::convert::From::from(c)),
                            Err(e) => {
                                error!("Something went weird while deregistering the connection: {:?}",
                                       e)
                            }
                        }                    }
                    None => error!("Could not remove connection from pool: {:?}", c),
                }
            },
            _ => {}
        }
    }
}
