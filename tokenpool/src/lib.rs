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
    
    pub fn pop(&mut self) -> Option<usize> {
        match self.queue.pop() {
            Some(n) => {
                Some(n.0)
            },
            
            None => {
                if self.floor < self.ceiling {
                    let n = self.floor;
                    self.floor += 1;
                    Some(n)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_handling() {
        let mut pool = TokenPool::new(0, 10);
        let p1 = pool.pop().unwrap();
        let p2 = pool.pop().unwrap();
        let p3 = pool.pop().unwrap();
        pool.push(p2);
        let p4 = pool.pop().unwrap();

        assert_eq!(p1, 0);
        assert_eq!(p3, 2);
        assert_eq!(p4, 1);
    }

    // Note that this is lossy: the tokens popped off here are not
    // recorded and can never be recovered.  This isn't a fatal error
    // (until you run out of tokens and you handle that poorly, of
    // course).

    #[test]
    fn exhausted_space() {
        let mut pool = TokenPool::new(0, 2);
        pool.pop();
        pool.pop();
        let p3 = pool.pop();
        assert_eq!(p3, None);
    }
}        
