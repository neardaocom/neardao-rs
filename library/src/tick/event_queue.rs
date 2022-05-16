use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

pub trait EventQueue<T> {
    fn unprocessed_len(&self) -> usize;
    fn add_event(&mut self, event: T);
    fn next_event(&mut self) -> Option<T>;
}

#[derive(BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
pub struct EventQueueVec<T> {
    queue: Vec<Option<T>>,
    next_pos: usize,
}

impl<T> EventQueueVec<T> {
    pub fn new() -> Self {
        Self {
            queue: Vec::default(),
            next_pos: 0,
        }
    }
}

impl<T> EventQueue<T> for EventQueueVec<T> {
    fn unprocessed_len(&self) -> usize {
        self.queue.len() - self.next_pos
    }
    fn add_event(&mut self, event: T) {
        self.queue.push(Some(event))
    }
    fn next_event(&mut self) -> Option<T> {
        self.next()
    }
}

impl<T> Iterator for EventQueueVec<T> {
    type Item = T;

    /// Yield `Event` and advances in queue.
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_pos < self.queue.len() {
            self.next_pos += 1;
            self.queue.get_mut(self.next_pos - 1).unwrap().take()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tick::event_queue::{EventQueue, EventQueueVec};

    #[derive(Debug, Clone, PartialEq)]
    enum Event {
        Group,
        Treasury,
        FunctionRole,
        Staking,
    }

    #[test]
    fn event_queue() {
        let mut q = EventQueueVec::new();
        q.add_event(Event::Treasury);
        q.add_event(Event::Group);
        q.add_event(Event::FunctionRole);
        q.add_event(Event::Staking);
        assert_eq!(q.unprocessed_len(), 4);
        let event = q.next_event().unwrap();
        assert_eq!(event, Event::Treasury);
        let event = q.next_event().unwrap();
        assert_eq!(event, Event::Group);
        assert_eq!(q.unprocessed_len(), 2);
        let event = q.next_event().unwrap();
        assert_eq!(event, Event::FunctionRole);
        let event = q.next_event().unwrap();
        assert_eq!(event, Event::Staking);
        assert_eq!(q.unprocessed_len(), 0);
        assert_eq!(q.next_event(), None);
    }
}
