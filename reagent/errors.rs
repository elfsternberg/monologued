// Create the Error, ErrorKind, ResultExt, and Result types

use std;
use std::io;
use error_chain::*;

error_chain! {
    foreign_links {
        Fmt(std::fmt::Error);
        Io(io::Error) #[cfg(unix)];
        SocketAddr(std::net::AddrParseError);
    }    
}
