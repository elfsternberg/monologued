use unicode_segmentation::UnicodeSegmentation;

use mio::tcp::TcpStream;
use mio::{Event, Token};
use mio::unix::UnixReady;
use std::io::{self, Read, Write};
use std::collections::VecDeque;
use bytes::{Buf, BufMut, BytesMut, Bytes};
use mio::Ready;


#[derive(PartialEq)]
pub enum ConnectionState {
    Running,
    Closed,
}

pub struct Connection {
    pub socket: TcpStream,
    buf: BytesMut,
    res: VecDeque<Bytes>,
    pub state: ConnectionState,
    pub token: Token,
    pub ready: Ready,
}

impl Connection {
    pub fn new(socket: TcpStream, token: Token) -> Self {
        Connection {
            socket: socket,
            buf: BytesMut::with_capacity(1024),
            res: VecDeque::new(),
            token: token,
            state: ConnectionState::Running,
            ready: Ready::readable() | Ready::from(UnixReady::hup()),
        }
    }

    
    
    // Take the event and decide what to do with it.
    pub fn message(&mut self, ready: &Ready) -> ConnectionState {
        if self.state == ConnectionState::Closed {
            return self.state;
        }

        if ready.is_writable() {
            self.write();
        }

        if ready.is_readable() {
            self.read();
        }
        self.state
    }

    // Notified that there is a desire to read, we read as much
    // as we can, copying the kernel buffer into the program
    // buffer (ugh), then notifying the protocol if there's
    // data to be read.  If anything goes wrong, we notify
    // the connection that something has failed.
    fn read(&mut self) {
        loop {
            let (len, ret) = {
                let mut buf = unsafe { &mut BytesMut::bytes_mut(&mut self.buf) };
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
                    self.state = ConnectionState::Closed;
                    return;
                }
            }
        }

        let c = self.buf.clone();
        loop {
            match c.iter().position(|&x| x == b'\n' || x == b'\r') {
                Some(p) => {
                    let word = String::from_utf8_lossy(&c[0..p]);
                    let drow: String = word.graphemes(true)
                        .rev()
                        .flat_map(|g| g.chars())
                        .collect();
                    self.res.push_back(Bytes::from(drow.into_bytes()));
                }
                None => {
                    break;
                }
            }
        }
        self.buf = BytesMut::with_capacity(512);
        self.buf.put_slice(&c[..]);
    }

    fn write(&mut self) {
        if self.res.len() == 0 {
            return;
        }

        loop {
            match self.res.pop_front() {
                Some(b) => {
                    loop {
                        let (len, res) = {
                            let len = b.len();
                            let res = self.socket.write(&b[..]);
                            (len, res)
                        };
                        match res {
                            Ok(0) => {
                                break;
                            }
                            Ok(r) => {
                                Buf::advance(&mut b, r);
                                if r != len || Buf::remaining_mut(b) == 0 {
                                    break;
                                }
                            }
                            Err(_) => {
                                Buf::advance(&mut b, len);
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
    }
}
