use std::collections::VecDeque;

pub struct RingBuffer<T> {
    capacity: usize,
    data: VecDeque<T>,
}

impl <T> RingBuffer<T> {
    pub fn new(capacity: usize) -> RingBuffer<T> {
        RingBuffer {
            capacity,
            data: VecDeque::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.data.push_back(item);
        if self.data.len() > self.capacity {
            self.data.pop_front();
        }
    }

    pub fn flush_vec(&mut self) -> Vec<T> {
        let deque = std::mem::replace(&mut self.data, VecDeque::new());
        Vec::from(deque)
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
