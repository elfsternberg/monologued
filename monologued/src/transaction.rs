
const MAX_BUF_SIZE: usize = 512;

enum TransactionError {
    BufferFull,
}

pub struct Transaction {
    // Handle to the socket we're talking with
    sock: TcpStream,

    // The token used to refer to this transaction.
    pub token: Token,

    // Set of events we're waiting for
    interest: Ready,

    // The buffer from which we'll be trying to derive the username
    buffer: BytesMut,

    // An optional buffer to the plan to output. (How do we look this up?)
    plan: Option<&Bytes>
}

const NO_PLAN: &'static str = "No plan found.";
const BAD_FORM: &'static str = "Request was not understood.";
const NO_FORWARD: &'static str = "This server does not support forwarding.";
const NO_LIST: &'static str = "This server does not support listing.";

impl Transaction {
    pub fn new(sock: TcpStream, token: Token) -> Transaction {
        Transaction {
            sock: sock,
            token: token,
            interest: Ready::from(UnixReady::hup()),
            buffer: BytesMut::with_capacity(MAX_BUF_SIZE),
        }
    }


    /// Register request to read with server
    ///
    /// Obviously, as this is single-threaded, this will hit the Poll
    /// before the next poll event.

    pub fn register(&mut self, poll: &mut Poll) -> io::Resul<()> {
        self.interest.insert(Ready::readable());
        poll.register(&self.sock, self.token, self.interest, PollOpt::edge())
            .or_else(|e| {
                error!("Failed to register {:?}, {:?}", self.token, e);
                Err(e)
            })
    }


    /// Attempt to read from the socket

    pub fn readable(&mut self) -> Result<()> {
        let (len, res) = {
            let mut buf = unsafe { &mut self.buf.bytes_mut() };
            let len = buf.len();
            let res = self.sock.read(buf);
            (len, res)
        };
        match res {
            Ok(0) => { res },
            Ok(r) => {
                unsafe { BufMut::advance(&mut self.buf, r); };
                if self.buf.iter(|c as char| c == '\r' || c == '\n') {
                    self.get_plan();
                    self.interest = Ready::writeable()
                }
                if !self.interest & Ready::writeable() && self.buf.remaining() == 0 {
                    Err(TransactionError::BufferFull)
                } else {
                    Ok(r)
                }
            },
            Err(e) => { res },
        }
    }

    /// Attempt to write to socket

    pub fn writeable(&mut self) -> Result<()> {
        let (len, res) = {
            let buf = &self.plan.bytes();
            let len = buf.len();
            let res = self.sock.write(buf);
        }
        match res {
            Ok(0) => { },
            Ok(r) => {
                Buf::advance(&mut self.plan, r);
                if Buf::remaining(&self.buf) == 0 {
                    self.interest = Ready::empty();
                }
            },
            Err(e) => { res }
        }
    }

    fn get_plan(&self) {
        
}
                
                
                

            
                    
                    
                    
                    
            
            
                
}
