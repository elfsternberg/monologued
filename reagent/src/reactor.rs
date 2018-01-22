/// At this point, you have a working queue.  Now to change the return type from
/// Option<> to Result<>, add all the networking stuff, and make the queue local
/// rather than inserted from the outside.

use std;
use errors::*;
use reagent::{ReAgent, Message};
use reagent::Message::*;
use mio::{Poll, Token, Event, Events};
use tokenpool::TokenPool;
use std::collections::{HashMap, VecDeque};


pub struct Reactor<'a> {
    poll: Poll,
    reagents: HashMap<Token, Box<ReAgent + 'a>>,
    tokens: TokenPool,
}

impl<'a> Reactor<'a> {

    pub fn new(max_connections: usize) -> Result<Self> {
        let poll = try!(Poll::new());
        Ok(Reactor {
            poll: poll,
            reagents: HashMap::new(),
            tokens: TokenPool::new(0, max_connections),
        })
    }

    pub fn add_agent(&mut self, mut reagent: Box<ReAgent>) -> Result<()> {
        if let Some(next_token) = self.tokens.pop() {
            let token = Token(next_token);
            reagent.set_token(token);
            reagent.register(&self.poll)?;
            self.reagents.insert(reagent.get_token(), reagent);
            return Ok(());
        }
        bail!(ErrorKind::ConnectionsExhausted)
    }

    pub fn rem_agent(&mut self, token: Token) -> Result<()> {
        match self.reagents.remove(&token) {
            Some(reagent) =>
                match reagent.deregister(&self.poll) {
                    Ok(_) => {
                        self.tokens.push(std::convert::From::from(reagent.get_token()));
                    },
                    Err(e) => {
                        panic!("Something went weird while deregistering the connection: {:?}", e);
                    }
                }
            None => panic!("Could not remove connection from pool: {:?}", token),
        }
        Ok(())
    }
                
    fn update_agent_poll_options(&mut self, token: Token) -> Result<()> {
        match self.reagents.get(&token) {
            Some(agent) => {
                agent.reregister(&self.poll)?;
            },
            None => {
                panic!("Failed to find connection during queue pass {:?}", token);
            }
        }
        Ok(())
    }
     
    fn handle_event(&mut self, event: &Event) -> Result<Message> {
        match self.reagents.get_mut(&event.token()) {
            Some(agent) => {
                agent.react(&event)
            }
            None => {
                panic!("Failed to find connection {:?}", event.token());
            }
        }
    }

    fn cycle(&mut self, messagequeue: &mut VecDeque<Message>) {
        while messagequeue.len() > 0 {
            match messagequeue.pop_front() {
                Some(message) => {
                    let _ = match message {   // TODO
                        AddAgent(agent) => self.add_agent(agent),
                        RemAgent(token) => self.rem_agent(token),
                        Reregister(token) => self.update_agent_poll_options(token),
                        //                            box PassMessage(message) => messagequeue.push_back(message),
                        Continue => Ok(()),
                    };
                }
                None => break
            };
        }
    }
    
    pub fn run(&mut self) -> Result<()> {
        let mut events = Events::with_capacity(self.reagents.len());

        loop {
            if self.reagents.len() < 1 {
                return Ok(())
            }

            match self.poll.poll(&mut events, None) {
                Ok(cnt) => {
                    let mut messagequeue = VecDeque::with_capacity(cnt);
                    for event in &events {
                        match self.handle_event(&event) {
                            Ok(message) => {
                                messagequeue.push_back(message);
                            }
                            Err(e) => {
                                panic!("Error while processing request: {:?}", e);
                            }
                        }
                    }
                    self.cycle(&mut messagequeue);
                }
                Err(e) => {
                    panic!("Unexpected error while polling: {:?}", e);
                }
            }
        }
    }
}

    
