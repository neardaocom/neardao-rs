use std::{collections::HashMap, hash::Hash};

use near_sdk::testing_env;

use crate::{
    event::{run_tick as tick, EventProcessor, EventQueue, TickEvent},
    unit_tests::{get_context_builder, get_default_contract},
    TimestampSec,
};

const EVENT_GROUP: &str = "group";
const EVENT_REWARDS: &str = "rewards";
const EVENT_ROLES: &str = "roles";

#[derive(Debug, Clone)]
enum Event {
    Group,
    Rewards,
    Roles,
}

impl Event {
    pub fn get_type(&self) -> String {
        match self {
            Event::Group => EVENT_GROUP,
            Event::Rewards => EVENT_REWARDS,
            Event::Roles => EVENT_ROLES,
        }
        .to_string()
    }
}

impl TickEvent for Event {}

#[derive(Debug)]
struct ProcessorMock {
    pub event_source: HashMap<TimestampSec, EventQueue<Event>>,
    pub next_tick: TimestampSec,
    pub tick_interval: TimestampSec,
    pub processing_results: Vec<String>,
}

impl ProcessorMock {
    pub fn new(
        event_source: HashMap<TimestampSec, EventQueue<Event>>,
        next_tick: TimestampSec,
        tick_interval: TimestampSec,
    ) -> Self {
        Self {
            event_source,
            next_tick,
            tick_interval,
            processing_results: vec![],
        }
    }

    pub fn assert_result(&self, expected_results: Vec<String>, msg: &str) {
        assert_eq!(
            self.processing_results, expected_results,
            "Processing results does not match: {}",
            msg
        )
    }

    pub fn assert_next_tick(&self, next_tick: TimestampSec) {
        assert_eq!(self.next_tick, next_tick);
    }

    pub fn update_event_source(
        &mut self,
        keys: Vec<TimestampSec>,
        event_source: HashMap<TimestampSec, EventQueue<Event>>,
    ) {
        let min = keys.iter().min().unwrap();
        self.next_tick = *min;
        for key in keys {
            self.event_source
                .insert(key, event_source.get(&key).unwrap().to_owned());
        }
    }
}

impl EventProcessor<Event> for ProcessorMock {
    fn get_next_tick(&self) -> TimestampSec {
        self.next_tick
    }

    fn tick_interval(&self) -> TimestampSec {
        self.tick_interval
    }

    fn process_event(&mut self, event: Event) {
        println!("processing event: {}", event.get_type());
        self.processing_results.push(event.get_type());
    }

    fn get_queue(&self, tick: TimestampSec) -> Option<EventQueue<Event>> {
        self.event_source.get(&tick).map(|e| e.to_owned())
    }

    fn remove_queue(&mut self, tick: TimestampSec) -> Option<EventQueue<Event>> {
        self.event_source.remove(&tick)
    }

    fn save_queue(
        &mut self,
        tick: TimestampSec,
        queue: &EventQueue<Event>,
    ) -> Option<EventQueue<Event>> {
        self.event_source.insert(tick, queue.to_owned())
    }

    fn set_next_tick(&mut self, tick: TimestampSec) {
        self.next_tick = tick
    }
}

macro_rules! make_test_data_and_result {
    ($event_source:expr, $expected_results: expr, $expected_count: literal, $($tick:literal => $($event:ident),*;)*) => {
        $(
            let mut queue = EventQueue::new();
            $(
                if $expected_results.len() < $expected_count {
                    $expected_results.push(Event::$event.get_type().to_string());
                }
                queue.add_event(Event::$event);
            )*
            $event_source.insert($tick,queue);
        )*
    };
}

#[test]
#[should_panic(expected = "not ready to tick")]
fn tick_not_ready() {
    let event_source = HashMap::new();
    let mut proc = ProcessorMock::new(event_source, 1, 10);
    tick(&mut proc, 10, 0);
}

#[test]
fn tick_process_one_and_half_queue() {
    let mut event_source = HashMap::new();
    let mut expected_results = vec![];

    make_test_data_and_result!(event_source, expected_results, 12,
        0 => Group, Group, Roles, Roles, Rewards, Rewards, Group, Group;
        10 => Roles,Roles, Rewards, Rewards, Group, Group;
    );

    let mut proc = ProcessorMock::new(event_source.clone(), 0, 10);

    let remaining = tick(&mut proc, 12, 20);
    assert_eq!(remaining, 2);
    proc.assert_result(expected_results, "queue 1: all");
    proc.assert_next_tick(10);
    assert!(proc.get_queue(0).is_none());
    assert_eq!(proc.get_queue(10).unwrap().unprocessed_len(), 2);
}

#[test]
fn tick_flow_scenario() {
    let mut event_source = HashMap::new();
    let mut expected_results = vec![];

    make_test_data_and_result!(event_source, expected_results, 14,
        0 => Group, Group, Roles, Roles, Rewards, Rewards, Group, Group;
        10 => Roles,Roles, Rewards, Rewards, Group, Group;
    );

    let mut proc = ProcessorMock::new(event_source.clone(), 0, 10);

    // Process first queue
    let remaining = tick(&mut proc, 2, 20);
    assert_eq!(remaining, 6);
    proc.assert_result(
        expected_results.clone().into_iter().take(2).collect(),
        "queue 1: 0-2",
    );
    proc.assert_next_tick(0);
    assert_eq!(proc.get_queue(0).unwrap().unprocessed_len(), 6);
    assert_eq!(proc.get_queue(10).unwrap().unprocessed_len(), 6);

    let remaining = tick(&mut proc, 2, 21);
    assert_eq!(remaining, 4);
    proc.assert_result(
        expected_results.clone().into_iter().take(4).collect(),
        "queue 1: 2-4",
    );
    proc.assert_next_tick(0);
    assert_eq!(proc.get_queue(0).unwrap().unprocessed_len(), 4);
    assert_eq!(proc.get_queue(10).unwrap().unprocessed_len(), 6);

    let remaining = tick(&mut proc, 4, 22);
    assert_eq!(remaining, 0);
    proc.assert_result(
        expected_results.clone().into_iter().take(8).collect(),
        "queue 1: 4-8",
    );
    proc.assert_next_tick(0);
    assert!(proc.get_queue(0).is_some());
    assert_eq!(proc.get_queue(10).unwrap().unprocessed_len(), 6);

    // Process next queue
    let remaining = tick(&mut proc, 4, 23);
    assert_eq!(remaining, 2);
    proc.assert_result(
        expected_results.clone().into_iter().take(12).collect(),
        "queue 2: 0-4",
    );
    proc.assert_next_tick(10);
    assert!(proc.get_queue(0).is_none());
    assert_eq!(proc.get_queue(10).unwrap().unprocessed_len(), 2);

    let remaining = tick(&mut proc, 2, 24);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "queue 2: 4-6");
    proc.assert_next_tick(10);
    assert!(proc.get_queue(0).is_none());
    assert_eq!(proc.get_queue(10).unwrap().unprocessed_len(), 0);

    // No tick is available
    let remaining = tick(&mut proc, 4, 30);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "no queue");
    proc.assert_next_tick(40);
    assert!(proc.get_queue(0).is_none());
    assert!(proc.get_queue(10).is_none());

    // No tick is available
    let remaining = tick(&mut proc, 4, 40);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "no queue");
    proc.assert_next_tick(40);
    assert!(proc.get_queue(0).is_none());
    assert!(proc.get_queue(10).is_none());

    // Tick available but time has not come yet
    let mut new_expected_results = vec![];
    make_test_data_and_result!(event_source, new_expected_results, 5,
        50 => Group, Group, Roles, Roles, Rewards;
    );
    expected_results.append(&mut new_expected_results);
    proc.update_event_source(vec![50], event_source);

    // Process last inserted tick - 50
    let remaining = tick(&mut proc, 5, 50);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "no queue");
    proc.assert_next_tick(50);
    assert!(proc.get_queue(0).is_none());
    assert!(proc.get_queue(10).is_none());
    assert_eq!(proc.get_queue(50).unwrap().unprocessed_len(), 0);

    // No tick is available
    let remaining = tick(&mut proc, 5, 60);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "no queue");
    proc.assert_next_tick(70);
    assert!(proc.get_queue(0).is_none());
    assert!(proc.get_queue(10).is_none());
    assert!(proc.get_queue(50).is_none());

    // No tick is available
    let remaining = tick(&mut proc, 5, 70);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results, "no queue");
    proc.assert_next_tick(70);
    assert!(proc.get_queue(0).is_none());
    assert!(proc.get_queue(10).is_none());
    assert!(proc.get_queue(50).is_none());
}
