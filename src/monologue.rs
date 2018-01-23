use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use mio::net::{TcpListener, TcpStream};
use mio::unix::UnixReady;
use mio::{Poll, PollOpt, Token, Ready, Event};
use reagent::reagent::{ReAgent, Message};
use reagent::errors::*;
use rfc1288::{Request, parse_rfc1288_request};
use std::io::{self, Read, Write};
use std::net::SocketAddr;


pub struct MonologueAgent {
    stream: TcpStream,
    pub token: Token,
    interest: Ready,
    buffer: BytesMut,
    response: BytesMut,
}

impl ReAgent for MonologueAgent {
    fn register(&self, poll: &Poll) -> Result<()> {
        poll.register(&self.stream, self.token, self.interest, PollOpt::edge())
            .chain_err(|| "Could not register client.")
    }

    fn reregister(&self, poll: &Poll)  -> Result<()> {
        poll.reregister(&self.stream, self.token, self.interest, PollOpt::edge())
            .chain_err(|| "Could not reregister client.")
    }

    fn deregister(&self, poll: &Poll)  -> Result<()> {
        poll.deregister(&self.stream)
            .chain_err(|| "Could not deregister client.")
    }

    fn get_token(&self) -> Token {
        self.token
    }

    fn set_token(&mut self, token: Token) {
        self.token = token;
    }

    fn react(&mut self, event: &Event) -> Result<Message> {
        if self.interest == Ready::empty() {
            return Ok(Message::RemAgent(self.token));
        }
        
        let readiness = event.readiness();
        
        if readiness.is_writable() {
            if let Some(message) = self.write() {
                return Ok(message);
            }
        }

        if readiness.is_readable() {
            if let Some(message) = self.read() {
                self.buffer.clear();
                return Ok(message);
            }
        }

        Ok(Message::Continue)
    }
}

impl MonologueAgent {
    pub fn new(stream: TcpStream) -> MonologueAgent {
        MonologueAgent {
            stream: stream,
            token: Token(0),
            interest: Ready::readable() | Ready::from(UnixReady::hup()),
            buffer: BytesMut::with_capacity(1024),
            response: BytesMut::new()
        }
    }

    fn read(&mut self) -> Option<Message> {
        loop {
            let (len, ret) = {
                let buf = unsafe { &mut BytesMut::bytes_mut(&mut self.buffer) };
                let len = buf.len();
                let ret = self.stream.read(buf);
                (len, ret)
            };
            match ret {
                Ok(0) => break,
                Ok(l) => {
                    unsafe {
                        self.buffer.advance_mut(l);
                    }
                    if l != len || BufMut::remaining_mut(&self.buffer) == 0 {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Socket is not ready anymore, stop accepting
                    break;
                }
                er => {
                    error!("Unexpected error: {:?}", er);
                    self.interest = Ready::empty();
                    return Some(Message::RemAgent(self.token));
                }
            }
        }

        let mut iter = self.buffer.split(|&x| x == b'\n' || x == b'\r');
        if let Some(command) = iter.next() {
            match parse_rfc1288_request(&Bytes::from(command)) {
                Ok(request) => {
                    match request {
                        Request::UserList => {
                            self.response.extend_from_slice(b"User list is not supported by this server\r\n");
                        }
                        Request::Remote(_, _) => {
                            self.response.extend_from_slice(b"Remote listing is not supported by this server\r\n");
                        }
                        Request::User(user) => {
                            let r = format!("The user you requested is {:?}\r\n", user).into_bytes();
                            self.response.extend_from_slice(&r);
                        }
                    }
                }
                Err(e) => {
                    self.response.extend_from_slice(&(format!("Error: {:?}\r\n", e).into_bytes()));
                }
            }
        }
        
        if self.response.len() > 0 {
            self.interest = Ready::readable() |
                            Ready::writable() |
                            Ready::from(UnixReady::hup());
            return Some(Message::Reregister(self.get_token()));
        }

        None
    }
    
    fn write(&mut self) -> Option<Message> {
        if self.response.len() == 0 {
            return None;
        }

        let len = self.response.len();
        let mut pos = 0;
        let mut buf = self.response.clone().into_buf();
        loop {
            let res = {
                self.stream.write(&Buf::bytes(&buf))
            };
            match res {
                Ok(0) => {
                    break;
                }
                Ok(r) => {
                    Buf::advance(&mut buf, r);
                    pos = pos + r;
                    if pos != len || Buf::remaining(&buf) == 0 {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Socket is not ready anymore, stop accepting
                    return Some(Message::Continue);
                }
                Err(_) => {
                    Buf::advance(&mut buf, len);
                    break;
                }
            }
        }
        self.interest = Ready::empty();
        Some(Message::RemAgent(self.get_token()))
    }
}

pub struct MonologueServer {
    listener: TcpListener,
    pub token: Token,
    interest: Ready,
}

impl MonologueServer {
    pub fn new(addr: &SocketAddr) -> Result<MonologueServer> {
        match TcpListener::bind(addr) {
            Ok(listener) => {
                Ok(MonologueServer {
                    listener: listener,
                    token: Token(0),
                    interest: Ready::readable(),
                })
            }
            
            // I thought about making this a chainable error, but no:
            // if a *server* doesn't configure, that's a crash-early
            // level mistake.
            Err(e) => {
                panic!("Could not bind to socket: {:?}", e);
            }
        }
    }
}

impl ReAgent for MonologueServer {
    fn register(&self, poll: &Poll) -> Result<()> {
        poll.register(&self.listener, self.token, self.interest, PollOpt::edge())
            .chain_err(|| "Could not register server.")
    }
    
    fn reregister(&self, poll: &Poll)  -> Result<()> {
        poll.reregister(&self.listener, self.token, self.interest, PollOpt::edge())
            .chain_err(|| "Could not reregister server.")
    }

    fn deregister(&self, poll: &Poll)  -> Result<()> {
        poll.deregister(&self.listener)
            .chain_err(|| "Could not deregister server.")
    }

    fn get_token(&self) -> Token {
        self.token
    }

    fn set_token(&mut self, token: Token) {
        self.token = token;
    }

    fn react(&mut self, event: &Event) -> Result<Message> {
        if event.readiness().is_readable() {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    Ok(Message::AddAgent(Box::new(MonologueAgent::new(stream))))
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        Ok(Message::Continue)
                    } else {
                        Err(::std::convert::From::from(e))
                    }
                }
            }
        } else {
            Ok(Message::Continue)
        }
    }
}

