use mio::{Token, Poll, Ready};
use std::io::{Result};

pub enum Message {
    AddAgent(Box<ReAgent>),
    RemAgent(Token),
    Reregister(Box<ReAgent>),
    PassMessage(Box<Message>),
    Continue,
}

pub trait ReAgent {
    fn register(&self, poll: &Poll) -> Result<()>;
    fn reregister(&self, poll: &Poll) -> Result<()>;
    fn deregister(&self, poll: &Poll) -> Result<()>;

    fn set_token(&mut self, Token);
    fn react(&self, event: Ready) -> Result<Message>;
}
