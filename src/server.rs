use tokenqueue::NextFree;
use connection::Connection;
use std::collections::HashMap;

use mio::*;
use std;
use mio::tcp::TcpListener;
use std::net::SocketAddr;
use std::io;


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
        info!("Starting server on address {:?} with token  {:?}", addr, token);
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
        loop {
            match self.poll.poll(&mut self.events, None) {
                Ok(cnt) => {
                    let mut i = 0;
                    while i < cnt {
                        let event = self.events.get(i);
                        match event {
                            Some(ev) => {
                                if ev.token() == self.token {
                                    self.accept();
                                } else {
                                    self.message(ev.token(), ev.readiness())
                                }
                            }
                            None => {
                                error!("Received event that was out of bound: {:?}", i);
                            }
                        }
                        i += 1;
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
                debug!("Token: {:?}, Connection Token: {:?}, client_addr: {:?}, socket: {:?}",
                       next_token, connection.token, &connection.socket, client_addr);
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
        let running = match self.connections.get_mut(&c) {
            Some(conn) => conn.message(&kind),
            None => {
                error!("Failed to look up connection {:?}", c);
                return;
            }
        };

        if running {
            match self.connections.get_mut(&c) {
                Some(connection) => {
                    match self.poll.reregister(&connection.socket,
                                               connection.token,
                                               connection.ready,
                                               PollOpt::edge()) {
                        Ok(_) => { },
                        Err(e) => {
                            error!("Error while reregistering connection {:?}: {:?}", &connection.token, e);
                        }
                    }
                }
                None => {
                    error!("Failed to look up connection {:?}", c);
                }
            }
        } else {
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
        }
    }
}
