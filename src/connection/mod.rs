use std::net::{SocketAddr};
use mio::tcp::{TcpStream};

pub struct Connection {
    pub client_addr: SocketAddr,
    pub handle: TcpStream,
}

impl Connection {
    pub fn new(handle: TcpStream, address: SocketAddr) -> Self {
        Connection {
            client_addr: address,
            handle: handle
        }
    }
}
