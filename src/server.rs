extern crate mio;
extern crate bytes;
    
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

enum ConnErrs {
    


struct Connection {
    client_addr: SocketAddr,
    handle: TcpStream,
    buffer: BytesMut,
    plan: Option<File>
}

impl Connection {
    fn new(handle: TcpStream, address: SocketAddr) -> Self {
        Connection {
            client_addr: address,
            handle: handle,
            buffer: BytesMut::with_capacity(MAX_LINE),
            plane: None
        }
    }
}

fn find_username(buffer: bytes::Buf) {
    Ok(req)
        .and_then(|req| {
            if req.len >= 2 { Ok(req.into_iter().peekable()) }
            else { Err(TransactionError::BadProtocol("Protocol prefix not recognized")) }
        })
        .and_then(|req| {
            if req.next() == Some('/') && req.next() == Some('W') { Ok(req) }
            else { Err(TransactionError::BadProtocol("Protocol prefix not recognized")) }
        })
        .and_then(|req| {
            if req.peek() == None || req.peek() != Some(' ') {
                Err(TransactionError::Unsupported("This server does not support user lists.")) }
            else { Ok(req) }
        })
        .and_then(|req| {
            while req.peek() == Some(' ') { req.next() };
            if req.peek() != None { Ok(req) } 
            else { Err(TransactionError::Unsupported("This server does not support user lists.")) }
        })
        .and_then(|req| {
            while is_unix_conventional(req.peek()) { reply.push(req.next()) };
            if req.peek() == '@' {
                Err(TransactionError::Unsupported("This server does not support remote queries")) }
            else { Ok(reply) }
        })
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
                    poll.register(&stream, Token(count), Ready::readable(), PollOpt::edge())
                        .unwrap();
                    let conn = Connection::new(stream, client_addr);
                    connections.insert(count, conn);
                }
                Token(c) => {
                    println!("Got token {}", c);

                    {
                        let conn = connections.get_mut(&c).unwrap();
                        if event.readiness().is_readable() {

                            let (len, res) = { 
                                let mut buf = unsafe { &mut conn.buffer.bytes_mut() };
                                let len = buf.len();
                                let res = conn.handle.read(buf);
                                (len, res)
                            };
                                    
                            match res {
                                Ok(0) => {
                                    Ok(0)
                                },
                                Ok(r) => {
                                    println!("Read {:?} bytes from {}", r, conn.client_addr);
                                    unsafe { conn.buffer.advance_mut(r) };
                                    match conn.buffer.into_iter().find(|c| c == 0xd || c == 0xa) {
                                        Some(r) {
                                            match find_username(conn.buffer.slice_to(r)) {
                                                Ok(username) => { },
                                                Err(e) => { },
                                            }


                                            
                                            let mut req = conn.buffer.slice_to(r);
                                            
                                                          
                                                    
                                                    
                                                    

                                                
                                                Ok
                                            

                                
                                    if r > 0 {
                                        poll.reregister(&conn.handle, Token(c), Ready::writable(), PollOpt::edge());
                                    }
                                }
                                Err(e) => panic!("What the heck? {:?}", e),
                            }
                        }
                            
                        if event.readiness().is_writable() {
                            let message = format!("Hello, {}, your message was {} long, you said: {:?}\r\n", conn.client_addr, conn.buffer.len(), &conn.buffer).into_bytes();
                            match conn.handle.write(&message) {
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
 
