use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::usize;

#[derive(Copy, Clone, Eq, PartialEq)]
struct NextFreeToken(usize);

// Reverses the ordering so that the *lowest* available token ID is returned.
impl Ord for NextFreeToken {
    fn cmp(&self, other: &NextFreeToken) -> Ordering {
        other.0.cmp(&self.0)
    }
}

// PartialOrd needs implementing, but this leverages the definition of
// cmp above and places it inside a moveable Option.
impl PartialOrd for NextFreeToken {
    fn partial_cmp(&self, other: &NextFreeToken) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct TokenPool {
    floor: usize,
    ceiling: usize,
    queue: BinaryHeap<NextFreeToken>,
}

impl TokenPool {
    pub fn new(floor: usize, ceiling: usize) -> Self {
        TokenPool {
            floor: floor,
            ceiling: ceiling,
            queue: BinaryHeap::new(),
        }
    }
    
    pub fn pop(&mut self) -> usize {
        match self.queue.pop() {
            Some(n) => {
                Ok(n.0)
            },
            
            None => {
                if (self.floor < self.ceiling) {
                    self.floor += 1;
                    Ok(self.floor)
                } else {
                    None
                }
            }
        }
    }

    pub fn push(&mut self, position: usize) {
        self.queue.push(NextFreeToken(position));
    }
}
