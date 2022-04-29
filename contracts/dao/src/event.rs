use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use crate::{core::Contract, TimestampSec};

pub trait TickEvent: Clone {}

pub trait EventProcessor<E: TickEvent> {
    fn get_next_tick(&self) -> TimestampSec;
    fn set_next_tick(&mut self, tick: TimestampSec);
    fn tick_interval(&self) -> TimestampSec;
    fn get_queue(&self, tick: TimestampSec) -> Option<EventQueue<E>>;
    fn remove_queue(&mut self, tick: TimestampSec) -> Option<EventQueue<E>>;
    fn save_queue(&mut self, tick: TimestampSec, queue: &EventQueue<E>) -> Option<EventQueue<E>>;
    fn process_event(&mut self, event: E);
}

/// Tick processing.
pub fn run_tick<E: TickEvent, C: EventProcessor<E>>(
    proc: &mut C,
    count: usize,
    current_timestamp: TimestampSec,
) -> usize {
    let mut next_tick = proc.get_next_tick();
    let tick_interval = proc.tick_interval();
    assert!(current_timestamp >= next_tick, "not ready to tick");

    let mut remaining = 0;
    let mut processed = 0;
    let mut event_queue = proc.remove_queue(next_tick);
    while let Some(mut queue) = event_queue.take() {
        remaining = queue.unprocessed_len();
        while let Some(event) = queue.next() {
            proc.process_event(event);
            processed += 1;
            remaining -= 1;
            if processed == count {
                break;
            }
        }
        if processed == count {
            proc.save_queue(next_tick, &queue);
            break;
        }
        while next_tick <= current_timestamp && event_queue.is_none() {
            next_tick += tick_interval;
            event_queue = proc.get_queue(next_tick);
        }
        proc.set_next_tick(next_tick);
    }
    remaining
}

#[allow(unused)]
impl EventProcessor<Event> for Contract {
    fn get_next_tick(&self) -> TimestampSec {
        todo!()
    }

    fn tick_interval(&self) -> TimestampSec {
        todo!()
    }

    fn get_queue(&self, tick: TimestampSec) -> Option<EventQueue<Event>> {
        todo!()
    }

    fn remove_queue(&mut self, tick: TimestampSec) -> Option<EventQueue<Event>> {
        todo!()
    }

    fn save_queue(
        &mut self,
        tick: TimestampSec,
        queue: &EventQueue<Event>,
    ) -> Option<EventQueue<Event>> {
        todo!()
    }

    fn set_next_tick(&mut self, tick: TimestampSec) {
        todo!()
    }

    fn process_event(&mut self, event: Event) {
        todo!()
    }
}

/// Event struct which is processed at a tick.
#[derive(BorshDeserialize, BorshSerialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub enum Event {
    Treasury,
    Group,
    FunctionRole,
    Staking,
}

impl TickEvent for Event {}

/// Event queue hold qeueu of events for a tick.
#[derive(BorshDeserialize, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Clone))]
pub struct EventQueue<T> {
    queue: Vec<Option<T>>,
    next_pos: usize,
}

impl<T> EventQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: Vec::default(),
            next_pos: 0,
        }
    }
    pub fn unprocessed_len(&self) -> usize {
        self.queue.len() - self.next_pos
    }

    pub fn add_event(&mut self, event: T) {
        self.queue.push(Some(event))
    }
}

impl<T> Iterator for EventQueue<T> {
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
