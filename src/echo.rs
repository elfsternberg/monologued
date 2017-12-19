use unicode_segmentation::UnicodeSegmentation;

use bytes::{Buf, BufMut, BytesMut, Bytes, IntoBuf};
use reagent::{ReAgent, Message};
use mio::net::{TcpListener, TcpStream};
use mio::unix::UnixReady;
use mio::{Poll, PollOpt, Token, Ready};
use std::collections::VecDeque;
use std::io::{self, Read, Write, Error};
use std::net::SocketAddr;

#[derive(PartialEq)]
pub enum ConnectionState {
    Running,
    Closed,
}

pub struct EchoAgent {
    stream: TcpStream,
    pub token: Token,
    interest: Ready,
    buffer: BytesMut,
    res: VecDeque<Bytes>,
}

impl ReAgent for EchoAgent {
    fn register(&self, poll: &Poll) -> io::Result<()> {
        poll.register(self.stream, self.interest, self.token, PollOpt::edge())
    }

    fn reregister(&self, poll: &Poll)  -> io::Result<()> {
        poll.reregister(self.stream, self.interest, self.token, PollOpt::edge())
    }

    fn deregister(&self, poll: &Poll)  -> io::Result<()> {
        poll.deregister(self.stream);
    }

    fn set_token(&mut self, token: Token) {
        self.token = token;
    }

    fn react(&self, &event: Ready) -> Result<Message, Error> {
        if self.interest == Ready::empty() {
            return Ok(Message::RemAgent(self.token));
        }

        if event.is_writeable() {
            if let Some(message) = self.write() {
                return Ok(message);
            }
        }

        if event.is_readable() {
            if let Some(message) = self.read() {
                return Ok(message);
            }
        }

        Ok()
    }
}

impl EchoAgent {
    fn new(&addr: SocketAddr) {
        let listener = try!(TcpListener::bind(addr));
        EchoServer {
            listener: listener,
            token: token,
            interest: Ready::readable(), // TODO: Unix HUP?
        }
    }
}    

    fn read(&mut self) {
        loop {
            let (len, ret) = {
                let buf = unsafe { &mut BytesMut::bytes_mut(&mut self.buf) };
                let len = buf.len();
                let ret = self.socket.read(buf);
                (len, ret)
            };
            match ret {
                Ok(0) => break,
                Ok(l) => {
                    unsafe {
                        self.buf.advance_mut(l);
                    }
                    if l != len || BufMut::remaining_mut(&self.buf) == 0 {
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

        let iter = self.buf.split(|&x| x == b'\n' || x == b'\r');
        for b in iter {
            let word = String::from_utf8_lossy(&b);
            let drow: String = word.graphemes(true)
                .rev()
                .flat_map(|g| g.chars())
                .collect();
            if drow.len() > 0 {
                self.res.push_back(Bytes::from(drow.into_bytes()));
            }
        }

        if self.res.len() > 0 {
            self.interest = Ready::readable() |
                            Ready::writable() |
                            Ready::from(UnixReady::hup());
            return Some(Message::Reregister(self));
        }

        None;
    }
    
    fn write(&mut self) {
        if self.res.len() == 0 {
            return;
        }

        loop {
            match self.res.pop_front() {
                Some(b) => {
                    let len = b.len();
                    let mut pos = 0;
                    let mut buf = b.into_buf();
                    loop {
                        let res = {
                            self.socket.write(&Buf::bytes(&buf))
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
                            Err(_) => {
                                Buf::advance(&mut buf, len);
                                break;
                            }
                        }
                    }
                }
                None => {
                    break;
                }

            }
        }
        self.interest = Ready::empty();
        Some(Message::RemAgent(self.token))
    }
}


pub struct EchoServer {
    listener: TcpListener,
    pub token: Token,
    interest: Ready,
}

impl EchoServer {
    fn new(&addr: SocketAddr) -> EchoServer {
        let listener = try!(TcpListener::bind(addr));
        EchoServer {
            listener: listener,
            token: token,
            interest: Ready::readable(), // TODO: Unix HUP?
        }
    }
}    


impl ReAgent for EchoServer {
    fn register(&self, poll: &Poll) -> io::Result<()> {
        poll.register(self.listener, self.interest, self.token, PollOpt::edge())
    }

    fn reregister(&self, poll: &Poll)  -> io::Result<()> {
        poll.reregister(self.listener, self.interest, self.token, PollOpt::edge())
    }

    fn deregister(&self, poll: &Poll)  -> io::Result<()> {
        poll.deregister(self.listener);
    }

    fn set_token(&mut self, token: Token) {
        self.token = token;
    }

    fn react(&self, event: Ready) -> Result<Message, Error> {
        match self.listener.accept() {
            Ok((stream, agent_addr)) => Ok(Message::AddAgent(EchoAgent::new(stream, agent_addr))),
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Message::Continue)
                } else {
                    Err(e)
                }
            }
        }
    }
}
