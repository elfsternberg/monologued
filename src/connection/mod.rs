use unicode_segmentation::UnicodeSegmentation;

use mio::tcp::TcpStream;
use mio::Token;
use mio::unix::UnixReady;
use std::io::{self, Read, Write};
use std::collections::VecDeque;
use bytes::{Buf, BufMut, BytesMut, Bytes, IntoBuf};
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
    pub fn message(&mut self, ready: &Ready) -> bool {
        if self.state == ConnectionState::Closed {
            return false;
        }

        if ready.is_writable() {
            self.write();
        }

        if ready.is_readable() {
            self.read();
        }

        ! (self.state == ConnectionState::Closed)
    }

    // Notified that there is a desire to read, we read as much
    // as we can, copying the kernel buffer into the program
    // buffer (ugh), then notifying the protocol if there's
    // data to be read.  If anything goes wrong, we notify
    // the connection that something has failed.
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
                    self.state = ConnectionState::Closed;
                    return;
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
            self.ready = Ready::readable() | Ready::writable() | Ready::from(UnixReady::hup());
        }
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
        self.state = ConnectionState::Closed;
    }
}
