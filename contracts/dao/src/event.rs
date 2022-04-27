use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub enum Event {
    Treasury,
    Group,
    FunctionRole,
    Staking,
}

#[derive(BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct EventQueue {
    queue: Vec<Option<Event>>,
    next_pos: usize,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            queue: Vec::default(),
            next_pos: 0,
        }
    }
    pub fn unprocessed_len(&self) -> usize {
        self.queue.len() - self.next_pos
    }

    pub fn add_event(&mut self, event: Event) {
        self.queue.push(Some(event))
    }
}

impl Iterator for EventQueue {
    type Item = Event;

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
    use super::{Event, EventQueue};

    #[test]
    fn event_queue() {
        let mut q = EventQueue::new();
        q.add_event(Event::Treasury);
        q.add_event(Event::Group);
        q.add_event(Event::FunctionRole);
        q.add_event(Event::Staking);
        assert_eq!(q.unprocessed_len(), 4);
        let event = q.next().unwrap();
        assert_eq!(event, Event::Treasury);
        let event = q.next().unwrap();
        assert_eq!(event, Event::Group);
        assert_eq!(q.unprocessed_len(), 2);
        let event = q.next().unwrap();
        assert_eq!(event, Event::FunctionRole);
        let event = q.next().unwrap();
        assert_eq!(event, Event::Staking);
        assert_eq!(q.unprocessed_len(), 0);
        assert_eq!(q.next(), None);
    }
}
