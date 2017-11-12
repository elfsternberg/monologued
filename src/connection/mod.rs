use unicode_segmentation::UnicodeSegmentation;

use mio::tcp::{TcpStream};
use mio::Event;
use std::io::{Read, Write};
use std::collections::VecDeque;
use bytes::{Buf, BufMut, BytesMut};
use mio::Ready;


pub enum ConnectionState {
    Running,
    Done
}

pub struct Connection {
    pub socket: TcpStream,
    buf: BytesMut,
    res: VecDeque<Vec<u8>>,
    done: bool,
    state: ConnectionState,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Connection {
            buf: BytesMut::with_capacity(512),
            res: VecDeque::new(),
            socket: socket,
            done: false,
            state: ConnectionState::Running,
        }
    }

    pub fn is_running(&self) -> bool {
        !self.done
    }

    // Take the event and decide what to do with it.
    pub fn handle(&mut self, event: &Event) -> bool {
        if self.is_running() && (event.readiness() & Ready::writable() == Ready::writable()) {
            self.write();
        }
        if self.is_running() && (event.readiness() & Ready::readable() == Ready::readable()) {
            self.read();
        }
        return self.is_running()
    }

    // Notified that there is a desire to read, we read as much
    // as we can, copying the kernel buffer into the program
    // buffer (ugh), then notifying the protocol if there's
    // data to be read.  If anything goes wrong, we notify
    // the connection that something has failed.
    fn read(&mut self) -> bool {
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
                    unsafe { self.buf.advance_mut(l); }
                    if l != len || BufMut::remaining_mut(&self.buf) == 0 {
                        break;
                    }
                },
                Err(e) => {
                    self.done = true;
                    return false;
                }
            }
        }
        let c = self.buf.clone();
        loop {
            match c.iter().position(|&x| x == b'\n' || x == b'\r') {
                Some(p) => {
                    let word = String::from_utf8_lossy(&c[0..p]);
                    let drow: String = word
                        .graphemes(true)
                        .rev()
                        .flat_map(|g| g.chars())
                        .collect();
                    self.res.push_back(drow.into_bytes());
                }
                None => { break; }
            }
        }
        true
    }

    fn write(&mut self) -> bool {
        if self.res.len() == 0 {
            return true
        }

        loop {
            let mut buf = self.res.pop_front();
            
            loop {
                
                let (len, res) = {
                    let ibuf = &buf.bytes();
                    let len = ibuf.len();
                    let res = self.socket.try_write(ibuf);
                    (len, res)
                };
                match res {
                    Ok(None) => {
                        break;
                    },
                    Ok(Some(r)) => {
                        Buf::advance(&mut buf, r);
                        if r != len || Buf::remaining_mut(buf) == 0 {
                            break;
                        }
                    },
                    Err(_) => {
                        Buf::advance(&mut buf, len);
                        break;
                    }
                }
            }
        }
    }
}
