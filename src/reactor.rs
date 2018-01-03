use tokenqueue::TokenPool;

use mio::*;
use std;
use std::io::{Result, Error, ErrorKind};
use std::collections::{VecDeque, HashMap};

pub use errors::*;

use reagent::{Message, ReAgent};

/* Server is a generalized single-pool abstraction around poll() */

pub struct Reactor<'a> {
    poll: Poll,
    agents: HashMap<Token, &'a ReAgent>,
    tokens: TokenPool,
}

impl<'a> Reactor<'a> {
    pub fn new(max_connections: usize) -> Result<Self> {
        let poll = try!(Poll::new());
        Ok(Reactor {
            poll: poll,
            agents: HashMap::new(),
            tokens: TokenPool::new(0, max_connections),
        })
    }

    pub fn add_agent<R>(&mut self, mut agent: R) -> Result<()>
        where R: ReAgent
    {
        if let Some(next_token) = self.tokens.pop() {
            agent.set_token(Token(next_token));
            agent.register(&self.poll);
            self.agents.insert(agent.get_token(), &agent);
            return Ok(())
        } 

        try!(Err("We are out of agent tokens?"))
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
        match self.agents.remove(&token) {
            Some(agent) =>
                match agent.deregister(&self.poll) {
                    Ok(_) => {
                        self.tokens.push(std::convert::From::from(&agent.get_token()))
                    },
                    Err(e) => {
                        error!("Something went weird while deregistering the connection: {:?}", e)
                    }
                }
            None => error!("Could not remove connection from pool: {:?}", token),
        }
    }
    
    fn handle_event(&mut self, event: &Event) -> Result<Message> {
        match self.agents.get_mut(&event.token()) {
            Some(agent) => {
                agent.react(&event)
            }
            None => {
                error!("Failed to find connection {:?}", event.token());
                Err(Error::new(ErrorKind::Other, "Failed to find connection."))
            }
        }
    }
    
    pub fn run(&mut self) -> Result<()> {
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
                        match self.handle_event(&event) {
                            Ok(message) => {
                                messagequeue.push_back(Box {message})
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
                    Some(message) => {
                        match message {
                            AddAgent(agent) => self.add_agent(agent),
                            RemAgent(token) => self.rem_agent(token),
                            Reregister(token) => self.update_agent_poll_options(token),
                            PassMessage(message) => messagequeue.push_back(message),
                            Continue => { },
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
}
