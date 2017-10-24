use rfc1288::{parse_rfc1288_request, Request};
use mio::*;
use std::time::{SystemTime};
use mio::tcp::{TcpStream};
use mio::unix::UnixReady;
use std::net::{SocketAddr};
use std::io::write;
use bytes::{Bytes, BytesMut, BufMut, Buf};
use std::*;

const MAX_BUF_SIZE: usize = 512;

enum TransactionError {
    BufferFull,
}

pub struct Transaction {
    // Handle to the socket we're talking with
    socket: TcpStream,

    // The address, for logging purposes
    pub client_addr: SocketAddr,
    
    // The token used to refer to this transaction.
    pub token: Token,

    // Set of events we're waiting for
    interest: Ready,

    // The buffer from which we'll be trying to derive the username
    buffer: BytesMut,

    // An optional buffer to the plan to output. (How do we look this up?)
    plan: Option<Bytes>,

    // When the transaction began.
    when: SystemTime
}

/* TODO: gettext() these messages */

const NO_PLAN: &'static str = "No plan found.";
const BAD_FORM: &'static str = "Request was not understood.";
const NO_FORWARD: &'static str = "This server does not support forwarding.";
const NO_LIST: &'static str = "This server does not support listing.";

impl Transaction {
    pub fn new(socket: TcpStream, address: SocketAddr, token: Token) -> Transaction {
        Transaction {
            socket: socket,
            client_addr: address,
            token: token,
            interest: Ready::from(UnixReady::hup()),
            buffer: BytesMut::with_capacity(MAX_BUF_SIZE),
            plan: None,
            when: SystemTime::now(),
        }
    }


    /// Register request to read with server
    ///
    /// Obviously, as this is single-threaded, this will hit the Poll
    /// before the next poll event.
    
    pub fn register_to_read(&mut self, poll: &mut Poll) -> io::Result<()> {
        self.interest.insert(Ready::readable());
        poll.register(&self.socket, self.token, self.interest, PollOpt::edge())
            .or_else(|e| {
                error!("Failed to register {:?}, {:?}", self.token, e);
                Err(e)
            })
    }
    

    fn get_plan(&mut self) {
        self.plan = match parse_rfc1288_request(&self.buffer) {
            Ok(r) => match r {
                Request::Remote(u, h) => {
                    Bytes::from("Remote Request: ".to_string() + u.to_string() + "@".to_string() + h.to_string())
                }
                Request::User(u) => {
                    Bytes::from("Local Request: ".to_string() + u.to_string())
                }
                Request::UserList => {
                    Bytes::from("List Request")
                }
            }, 
            Err(e) => {
                Bytes::from(e.description())
            }
        }
    }

    /// Attempt to read from the socket
    pub fn read(&mut self, poll: &mut Poll) -> Result<usize, std::io::Error> {
        let (len, res) = {
            let mut buf = unsafe { &mut self.buffer.bytes_mut() };
            let len = buf.len();
            let res = self.socket.read(buf);
            (len, res)
        };
        match res {
            Ok(0) => { res },
            Ok(r) => {
                unsafe { BufMut::advance(&mut self.buffer, r); };
                if self.buffer.iter(|c: char| c == b'\r' || c == b'\n') {
                    self.interest = (self.interest | Ready::writeable()) & !Ready::readable();
                    self.get_plan();
                }
                if !self.interest & Ready::writeable() && self.buffer.remaining() == 0 {
                    Err(TransactionError::BufferFull)
                } else {
                    if self.interest & Ready::writeable() {
                        poll.reregister(&self.socket, self.token, self.interest, PollOpt::edge())
                            .or_else(|e| {
                                error!("Failed to register {:?}, {:?}", self.token, e);
                                Err(e)
                            })
                    } else {
                        Ok(r)
                    }
                }
                
            },
            Err(e) => { res },
        }
    }

    /// Attempt to write to socket

    pub fn write(&mut self, poll: &mut Poll) -> Result<usize, std::io::Error> {
        let (len, res) = {
            let buf = &self.plan.bytes();
            let len = buf.len();
            let res = self.socket.write(buf);
            (len, res)
        };
        match res {
            Ok(0) => { Ok(0) },
            Ok(r) => {
                Buf::advance(&mut self.plan, r);
                if Buf::remaining(&self.buffer) == 0 {
                    self.interest = Ready::empty();
                }
                if self.interest == Ready::empty() {
                    poll.deregister(&self.socket);
                }
                Ok(r)
            },
            Err(e) => { res }
        }
    }
}
                
