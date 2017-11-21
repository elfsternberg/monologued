struct Server {
    socket: TcpListener,
    connections: Slab<Connection>,
    events: Events,
    poll: Poll,
    count: usize,
}

impl Server {
    pub fn new(addr: SocketAddr, token: Token, max_connections: usize) -> io::Result<Server> {
        let socket = try!(TcpListener::bind(&addr));
        let poll = try!(Poll::new().unwrap());
        try!(poll.register(&sock, token, Ready::readable(), PollOpt::edge()));
        
        info!("Starting server on address {:?}", addr);
        Ok(Server {
            socket: socket,
            connections: Slab::new_starting_at(token, max_connections),
            events: Events::with_capacity(max_connections),
            poll: poll,
            count: token.from(),
        })
    }

    pub fn run(&mut self) {
        loop {
            match self.poll.poll(&mut events, None) {
                Ok(event_count) => {
                    for event in events.iter() {
                        match event.token() {
                            self.token => {
                                self.accept()
                            }
                            
                            Token(c) => {
                                self.message(c)
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error while polling: {:?}", e);
                }
            }
        }
    }

    pub fn accept(&mut self) {
        match listener.accept() {
            Ok(socket, client_addr) => {
                self.count += 1;
                info!("Connection from {:?}", client_addr);
                connection = Connection::new(&socket, Token(count));
                match poll.register(&connection.socket(),
                                    &connection.token(),
                                    &connection.ready(),
                                    PollOpt::edge()) {
                    Ok(_) => {
                        self.connections.insert(count, connection);
                    }
                    Err(e) => {
                        error!("Error while adding connection: {:?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Error while accepting connection: {:?}", e);
            }
        }
    }
        
    pub fn message(&mut self, c: Token) {
        
                    
                        
            
    
}

