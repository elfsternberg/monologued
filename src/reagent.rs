use mio::{Token, Poll, Event};
use errors::*;

// All this boxing is necessary in order to make Message movable and
// queue-able, as a Box sets an upper limit on the size of the Message
// itself

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

    fn get_token(&mut self) -> Token;
    fn set_token(&mut self, Token);
    fn react(&mut self, event: &Event) -> Result<Message>;
}
