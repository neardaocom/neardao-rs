use crate::TimestampSec;

use super::{
    event_processor::{EventProcessor, TickEvent},
    event_queue::EventQueue,
};

/// Event queue processing.
/// Processing stops when `count` of events has been processed
/// or all queues up to `current_timestamp` has been processed.
/// Tick saves timestamp of last fully processed queue as last_tick.
/// Queue at last_tick is removed.
/// Returns remaining size of unprocessed events in last processed queue.
///
/// Processing Invariants:
/// 1. Everything before last_tick (included) must be processed.
pub fn run_tick<E, Q, C>(proc: &mut C, count: usize, current_timestamp: TimestampSec) -> usize
where
    E: TickEvent,
    Q: EventQueue<E>,
    C: EventProcessor<E, Q>,
{
    let tick_interval = proc.tick_interval();
    let last_tick = proc.get_last_tick();
    let mut next_tick = last_tick + tick_interval;
    assert!(current_timestamp >= next_tick, "not ready to tick");

    let mut remaining = 0;
    let mut processed = 0;
    while next_tick <= current_timestamp {
        if let Some(mut queue) = proc.remove_queue(next_tick) {
            remaining = queue.unprocessed_len();
            while let Some(event) = queue.next_event() {
                proc.process_event(event);
                processed += 1;
                remaining -= 1;
                if processed == count {
                    if remaining > 0 {
                        proc.save_queue(next_tick, &queue);
                        proc.set_last_tick(next_tick - tick_interval);
                    } else {
                        proc.set_last_tick(next_tick);
                    }
                    return remaining;
                }
            }
        }
        next_tick += tick_interval;
    }
    proc.set_last_tick(next_tick - tick_interval);
    remaining
}
