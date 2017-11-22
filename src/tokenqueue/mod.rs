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

pub struct NextFree {
    max: usize,
    queue: BinaryHeap<NextFreeToken>,
}

impl NextFree {
    pub fn new(start: usize) -> Self {
        NextFree {
            max: start,
            queue: BinaryHeap::new(),
        }
    }

    pub fn pop(&self) -> usize {
        match self.queue.peek() {
            Nothing => {
                self.max += 1;
                self.max
            }

            Some(n) => self.queue.pop().unwrap().0,
        }
    }

    pub fn push(&self, position: usize) {
        self.queue.push(NextFreeToken(position));
    }
}
