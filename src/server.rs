extern crate mio;

use std::io::{Read, Write};
use mio::tcp::{TcpListener, TcpStream};
use mio::{Token, Ready, Events, Poll, PollOpt};
use mio::timer::Timer;
use std::collections::HashMap;
use std::time::Duration;


struct Monologue {
    buffer: [u8; 512],
    handle: TcpStream
}

impl Monologue {
    fn new(handle: TcpStream) -> Self {
        Monologue {
            buffer: [0; 512],
            handle: handle
        }
    }
}


const MONOLOGUE: Token = Token(0);
const TIMER: Token = Token(1);
const ADDR: &'static str = "0.0.0.0:6667";


struct Server {
    addr: SocketAddr,
    server: TcpListener,
    poll: Poll
}

impl Server {
    pub fn new(addr: &SocketAddr, logger: &Logger) {
        Server {
            server: TcpListener::bind(&addr).unwrap(),
            poll: Poll::new().unwrap();
        }
    }

    pub fn serve(self) {
        let mut buf = [0; 1024];
        let mut events = Events::with_capacity(1024);
        let mut connections = HashMap::new();
        let mut count = 1;

        poll.register(&server, MONOLOGUE, Ready::readable(), PollOpt::edge()).unwrap();
        poll.register(&timer, TIMER, Ready::readable(), PollOpt::edge()).unwrap();

        let flush_timeout = Duration::from_millis(15000);
        timer.set_timeout(flush_timeout, String::from("hello from 5s ago")).ok();

        loop {
            self.poll.poll(&mut events, Note).unwrap();
            for event in events.iter() {
                match event.token() {
                    MONOLOGUE =>
                        match server.accept() {
                            Err(e) => logger.log(e),
                            OK((stream, client_addr)) => {
                                count += 1;
                                poll.register(&stream, Token(count), Ready::readable(), PollOpt::edge()).unwrap();
                                let conn = Monologue::new(stream);
                                connections.insert(count, conn);
                            }
                        }

                    TIMER => {
                        println!("Number of connections: {}", connections.len());
                        timer.set_timeout(flush_timeout, String::from("hello from 5s ago")).ok();
                    }

                    Token(c) => {
                        let conn = connections.get_mut(&c).unwrap();
                        n = match conn.handle.read(&mut buf) {
                            Ok(m) => m,
                            Err(e) => panic!("Got an error while reading from a connection: {}", e)
                        };
                        
                        if n != 0 {
                            println!("Read {} bytes from client", n);
                            match conn.handle.write(&buf[..n]) {
                                Ok(n) => println!("Wrote {} bytes to client", n),
                                Err(e) => println!("Got an error while writing to th connection: {}", e)
                            };
                        }
                    }
                        
                    if n == 0 {
                        // EOF, Client closed connection
                        let conn = connections.remove(&c).unwrap();
                        poll.deregister(&conn.handle);
                    }
                }
                        
                    
        


fn main() {
    let address = ADDR.parse().unwrap();
    let server = TcpListener::bind(&address).unwrap();
    let poll = Poll::new().unwrap();
    
    let mut buf = [0; 512];
    let mut events = Events::with_capacity(1024);
    let mut timer = Timer::default();
    
    poll.register(&server, MONOLOGUE, Ready::readable(), PollOpt::edge()).unwrap();

    let flush_timeout = Duration::from_millis(5000);
    timer.set_timeout(flush_timeout, String::from("hello from 5s ago")).ok();

    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                MONOLOGUE => {
                    let (stream, client_addr) = match server.accept() {
                        Ok((stream, client_addr)) => (stream, client_addr),
                        Err(e) => panic!("Got an error while handling a connection.")
                    };
                    count += 1;
                    poll.register(&stream, Token(count), Ready::readable(), PollOpt::edge())
                        .unwrap();
                    let conn = Monologue::new(stream);
                    connections.insert(count, conn);
                }
                TIMER => {
                    println!("Number of connections: {}", connections.len());
                    timer.set_timeout(flush_timeout, String::from("hello from 5s ago")).ok();
                }
                Token(c) => {
            }
        }
    }
}

