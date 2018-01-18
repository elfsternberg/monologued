use tokenpool::TokenPool;

use std;
use mio::{Poll, Token, Event, Events};
use std::collections::{VecDeque, HashMap};

use errors::*;

use reagent::{ReAgent, Message};
use reagent::Message::*;

/// A Reactor has a handle on a poll, a list of Reagents that it cares
/// about, and a TokenPool that it uses to track the number of
/// connection IDs in use and currently available.

pub struct Reactor<'a> {
    poll: Poll,
    agents: HashMap<Token, Box<ReAgent + 'a>>,
    tokens: TokenPool,
}


impl<'a> Reactor<'a> {

    /// Constructor. Derp.
    
    pub fn new(max_connections: usize) -> Result<Self> {
        let poll = try!(Poll::new());
        Ok(Reactor {
            poll: poll,
            agents: HashMap::new(),
            tokens: TokenPool::new(0, max_connections),
        })
    }

    /// Given a Reagent, try to add it to the system.  Huh... should I
    /// check if the token is free / has been freed before requesting
    /// it?
    
    pub fn add_agent<R>(&mut self, mut agent: R) -> Result<()>
        where R: Box<ReAgent + 'a>
    {
        if let Some(next_token) = self.tokens.pop() {
            agent.set_token(Token(next_token));
            agent.register(&self.poll);
            self.agents.insert(agent.get_token(), agent);
            return Ok(())
        } 

        bail!(ErrorKind::ConnectionsExhausted)
    }

    /// ReAgent.ReRegister() handler...
    
    fn update_agent_poll_options(&mut self, token: Token) -> Result<()> {
        match self.agents.get(&token) {
            Some(agent) => {
                agent.reregister(&self.poll);
            },
            None => {
                panic!("Failed to find connection during queue pass {:?}", token);
            }
        }
        Ok(())
    }

    /// ReAgent.DeRegister() handler...
    
    pub fn rem_agent(&mut self, token: Token) -> Result<()> {
        match self.agents.remove(&token) {
            Some(agent) =>
                match agent.deregister(&self.poll) {
                    Ok(_) => {
                        self.tokens.push(std::convert::From::from(agent.get_token()))
                    },
                    Err(e) => {
                        panic!("Something went weird while deregistering the connection: {:?}", e)
                    }
                }
            None => panic!("Could not remove connection from pool: {:?}", token),
        }
        Ok(())
    }

    /// React for one agent.
    
    fn handle_event(&mut self, event: &Event) -> Result<Message> {
        match self.agents.get_mut(&event.token()) {
            Some(agent) => {
                agent.react(&event)
            }
            None => {
                panic!("Failed to find connection {:?}", event.token());
            }
        }
    }

    /// Send reactions to events.
    
    pub fn run(&mut self) -> Result<()> {
        let mut events = Events::with_capacity(self.agents.len());
        let mut messagequeue = VecDeque::with_capacity(events.len());
        
        loop {
            if self.agents.len() < 1 {
                // Possible if we're done.
                return Ok(())
            }
        
            match self.poll.poll(&mut events, None) {
                Ok(cnt) => {
                    for event in events {
                        match self.handle_event(&event) {
                            Ok(message) => {
                                messagequeue.push_back(message)
                            }
                            Err(e) => {
                                panic!("Error while processing request: {:?}", e)
                            }
                        }
                    }
                }
            }

            while messagequeue.len() > 0 {
                match messagequeue.pop_front() {
                    Some(message) => {
                        match message {
                            AddAgent(agent) => self.add_agent(agent),
                            RemAgent(token) => self.rem_agent(token),
                            Reregister(token) => self.update_agent_poll_options(token),
//                            box PassMessage(message) => messagequeue.push_back(message),
                            Continue => Ok(()),
                        };
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
}
