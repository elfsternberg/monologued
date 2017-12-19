use tokenqueue::TokenPool;

use mio::*;
use std;
use std::io::{Result, Error, ErrorKind};
use std::collections::{VecDeque, HashMap};

use reagent::{Message, ReAgent};

/* Server is a generalized single-pool abstraction around poll() */

pub struct Reactor {
    poll: Poll,
    agents: HashMap<Token, Box<ReAgent>>,
    tokens: TokenPool,
}

impl Reactor {
    pub fn new(max_connections: usize) -> Self {
        let poll = try!(Poll::new());
        Reactor {
            poll: poll,
            tokens: TokenPool::new(max_connections),
        }
    }

    /* This feels a bit like cheating.  The ?Sized attribute here is
       more or less an 'Any' stand-in, where we then bind the 'R' to
       the method by the 'where' clause.
     */
    
    pub fn add_agent<R: ?Sized>(&mut self, mut agent: R)
        where R: ReAgent
    {
        agent.token(self.tokens.pop());
        agent.register(&self.poll);
        self.agents.insert(&agent.token, &agent);
    }

    fn update_agent_poll_options(&mut self, token: Token) {
        match self.agents.get(token) {
            Some(agent) => {
                agent.reregister(&self.poll);
            },
            None => {
                error!("Failed to find connection during queue pass {:?}", token);
            }
        }
    }

    pub fn rem_agent(&mut self, token: Token) {
        match self.agents.remote(&token) {
            Some(agent) =>
                match agent.deregister(&self.poll) {
                    Ok(_) => { self.tokens.push(std::convert::From::from(&agent.token)) },
                    Err(e) => {
                        error!("Something went weird while deregistering the connection: {:?}", e)
                    }
                }
            None => error!("Could not remove connection from pool: {:?}", token),
        }
    }
    
    fn handle_event(&mut self, &event: Event) -> Result<Message> {
        match self.agents.get_mut(event.token()) {
            Some(agent) => {
                agent.react(event)
            }
            None => {
                error!("Failed to find connection {:?}", event.token());
                Error::new(ErrorKind::other, "Failed to find connection.")
            }
        }
    }
    
    pub fn run(&mut self) {
        use reagent::Message::*;

        if self.agents.len() < 1 {
            panic!("The server cannot be run on an empty collection.")
        }
        
        let mut events = Events::with_capacity(self.agents.len());
        let mut messagequeue = VecDeque::with_capacity(events.len());
        
        loop {
            match self.poll.poll(&mut events, None) {
                Ok(cnt) => {
                    for event in events {
                        match self.handle_event(event) {
                            Ok(message) => {
                                messagequeue.push_back(self.handle_event(event))
                            }
                            Err(e) => {
                                error!("Error while processing request: {:?}", e)
                            }
                        }
                    }
                }
            }
            while messagequeue.len() > 0 {
                match messagequeue.pop_front() {
                    AddAgent(agent) => self.add_agent(agent),
                    RemAgent(token) => self.rem_agent(token),
                    Reregister(token) => self.update_agent_poll_options(token),
                    PassMessage(message) => messagequeue.push_back(message),
                    Continue => { },
                }
            }
        }
    }
}
