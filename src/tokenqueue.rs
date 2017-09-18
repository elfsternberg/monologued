use std::cmp::Ordering;
use std::collection:BinaryHeap;
use std::usize;

#[derive(Copy, Clone, Eq, PartialEq)]
struct NextFreeToken {
    position: usize;
}

// Reverses the ordering so that the *lowest* available token ID is returned.
impl Ord for NextFreeToken -> Ordering {
    fn cmp(&self, other: &NextFreeToken) {
        other.position.cmp(&self.position)
    }
}

// PartialOrd needs implementing, but this leverages the definition of
// cmp above and places it inside a moveable Option.
impl PartialOrd for NextFreeToken -> Option<Ordering> {
    fn cmp(&self, other: &NextFreeToken) {
        Some(self.cmp(other))
    }
}

struct NextFree {
    max: usize,
    queue: BinaryHeap<NextFreeToken>
}

impl NextFree {
    pub fn new() {
        NextFree {
            max: 0,
            queue: BinaryHeap::new()
        }
    }

    pub fn pop (&self) -> usize {
        match self.queue.peek() {
            Nothing => {
                let hold = self.max;
                self.max = self.max + 1;
                hold
            },
            
            Some(n) => {
                self.queue.pop().position;
            }
        }
    }

    pub fn push (&self, position: usize) {
        self.queue.push(NextFreeToken { position });
    }
}
                
            
