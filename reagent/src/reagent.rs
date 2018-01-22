use mio::{Token, Poll, Event};
use errors::*;


pub enum Message {
    AddAgent(Box<ReAgent>),
    RemAgent(Token),
    Reregister(Token),
    Continue,
}

pub trait ReAgent {
    // The Reactor registers the ReAgent with the Reactor.poll
    fn register(&self, poll: &Poll) -> Result<()>;
    fn reregister(&self, poll: &Poll) -> Result<()>;
    fn deregister(&self, poll: &Poll) -> Result<()>;

    // The Reactor gets and sets the token for this ReAgent.
    fn get_token(&self) -> Token;
    fn set_token(&mut self, Token);

    // The Reactor sends a message to the ReAgent telling it a useful event has occured.
    fn react(&mut self, event: &Event) -> Result<Message>;
}
