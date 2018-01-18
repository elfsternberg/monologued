use mio::{Token, Poll, Event};
use errors::*;

// All this boxing is necessary in order to make Message movable and
// queue-able, as a Box sets an upper limit on the size of the Message
// itself

/// The Message type used by Reactor and Reagents to communicate
/// between one another.
///
/// AddAgent - Reagents which may build new connections pass bad
/// AddAgent() messages with the agent to be added.  It is the
/// Reactor's job to call add_agent(), assigning an internal
/// connection id (a mio Token in this case) and to put the new agent
/// into the queue.
///
/// RemAgent - Agents which have completed their duties return this at
/// the end of processing to tell the reactor it needs to be shut down
/// cleanly.
///
/// Reregister - Agents which need to change their Interest return
/// this at the end of a reaction to inform the Reactor of the change.
///
/// PassMessage - Currently unused, but this can be used to pass
/// messages from one ReAgent to another.  Right now this is pretty
/// abstract.
///
/// Continue - NoOp.  Do nothing; keep processing.

pub enum Message {
    AddAgent(Box<ReAgent>),
    RemAgent(Token),
    Reregister(Token),
//    PassMessage(Box<Message>),
    Continue,  // This is mostly a no-op
}

/// The ReAgent Type
///
/// Register, Deregister, and Poll are all more or less what you'd
/// expect.  They're constructed this way to make the interface clean.
/// I'm not sure this is the Rustlang way to do it, but it passes, so
/// I can't complain.

pub trait ReAgent {
    // The Reactor registers the ReAgent with the Reactor.poll
    fn register(&self, poll: &Poll) -> Result<()>;
    fn reregister(&self, poll: &Poll) -> Result<()>;
    fn deregister(&self, poll: &Poll) -> Result<()>;

    // The Reactor gets and sets the token for this ReAgent.
    fn get_token(&mut self) -> Token;
    fn set_token(&mut self, Token);

    // The Reactor sends a message to the ReAgent telling it a useful event has occured.
    fn react(&mut self, event: &Event) -> Result<Message>;
}
