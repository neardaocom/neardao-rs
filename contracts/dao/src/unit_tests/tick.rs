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
    pub last_tick: TimestampSec,
    pub tick_interval: TimestampSec,
    pub processing_results: Vec<String>,
}

impl ProcessorMock {
    pub fn new(
        event_source: HashMap<TimestampSec, EventQueue<Event>>,
        last_tick: TimestampSec,
        tick_interval: TimestampSec,
    ) -> Self {
        Self {
            event_source,
            last_tick,
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

    pub fn assert_last_tick(&self, last_tick: TimestampSec) {
        assert_eq!(self.last_tick, last_tick);
    }

    pub fn update_event_source(
        &mut self,
        keys: Vec<TimestampSec>,
        event_source: HashMap<TimestampSec, EventQueue<Event>>,
    ) {
        for key in keys {
            self.event_source
                .insert(key, event_source.get(&key).unwrap().to_owned());
        }
    }
}

impl EventProcessor<Event> for ProcessorMock {
    type ProcessingResult = ();

    fn get_last_tick(&self) -> TimestampSec {
        self.last_tick
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

    fn set_last_tick(&mut self, tick: TimestampSec) {
        self.last_tick = tick
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
fn tick_process_part_queue() {
    let mut event_source = HashMap::new();
    let mut expected_results = vec![];

    make_test_data_and_result!(event_source, expected_results, 4,
        10 => Group, Group, Roles, Roles, Rewards, Rewards, Group, Group;
        20 => Roles,Roles, Rewards, Rewards, Group, Group;
    );

    let mut proc = ProcessorMock::new(event_source.clone(), 0, 10);

    let remaining = tick(&mut proc, 4, 20);
    assert_eq!(remaining, 4);
    proc.assert_result(expected_results, "part queue");
    proc.assert_last_tick(0);
    assert!(proc.get_queue(0).is_none());
    assert_eq!(proc.get_queue(10).unwrap().unprocessed_len(), 4);
}

#[test]
fn tick_process_one_queue_exact() {
    let mut event_source = HashMap::new();
    let mut expected_results = vec![];

    make_test_data_and_result!(event_source, expected_results, 8,
        10 => Group, Group, Roles, Roles, Rewards, Rewards, Group, Group;
        20 => Roles,Roles, Rewards, Rewards, Group, Group;
    );

    let mut proc = ProcessorMock::new(event_source.clone(), 0, 10);

    let remaining = tick(&mut proc, 8, 30);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results, "queue one all");
    proc.assert_last_tick(10);
    assert!(proc.get_queue(10).is_none());
    assert_eq!(proc.get_queue(20).unwrap().unprocessed_len(), 6);
}

#[test]
fn tick_process_one_queue_available() {
    let mut event_source = HashMap::new();
    let mut expected_results = vec![];

    make_test_data_and_result!(event_source, expected_results, 8,
        10 => Group, Group, Roles, Roles, Rewards, Rewards, Group, Group;
        20 => Roles,Roles, Rewards, Rewards, Group, Group;
    );

    let mut proc = ProcessorMock::new(event_source.clone(), 0, 10);

    let remaining = tick(&mut proc, 8, 15);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results, "queue one all");
    proc.assert_last_tick(10);
    assert!(proc.get_queue(10).is_none());
    assert_eq!(proc.get_queue(20).unwrap().unprocessed_len(), 6);
}

#[test]
fn tick_process_one_and_half_queue() {
    let mut event_source = HashMap::new();
    let mut expected_results = vec![];

    make_test_data_and_result!(event_source, expected_results, 12,
        10 => Group, Group, Roles, Roles, Rewards, Rewards, Group, Group;
        20 => Roles,Roles, Rewards, Rewards, Group, Group;
    );

    let mut proc = ProcessorMock::new(event_source.clone(), 0, 10);

    let remaining = tick(&mut proc, 12, 40);
    assert_eq!(remaining, 2);
    proc.assert_result(expected_results, "queue 1: all");
    proc.assert_last_tick(10);
    assert!(proc.get_queue(10).is_none());
    assert_eq!(proc.get_queue(20).unwrap().unprocessed_len(), 2);
}

#[test]
fn tick_process_two_queues_with_gaps() {
    let mut event_source = HashMap::new();
    let mut expected_results = vec![];

    make_test_data_and_result!(event_source, expected_results, 14,
        30 => Group, Group, Roles, Roles, Rewards, Rewards, Group, Group;
        60 => Roles,Roles, Rewards, Rewards, Group, Group;
    );

    let mut proc = ProcessorMock::new(event_source.clone(), 0, 10);

    let remaining = tick(&mut proc, 20, 101);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results, "queue 1: all");
    proc.assert_last_tick(100);
    assert!(proc.get_queue(30).is_none());
    assert!(proc.get_queue(60).is_none());
}

#[test]
fn tick_flow_scenario() {
    let mut event_source = HashMap::new();
    let mut expected_results = vec![];

    make_test_data_and_result!(event_source, expected_results, 14,
        10 => Group, Group, Roles, Roles, Rewards, Rewards, Group, Group;
        20 => Roles,Roles, Rewards, Rewards, Group, Group;
    );

    let mut proc = ProcessorMock::new(event_source.clone(), 0, 10);
    proc.assert_last_tick(0);

    // 1. Process both queues
    let remaining = tick(&mut proc, 20, 20);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "flow_scenario - 1");
    proc.assert_last_tick(20);
    assert!(proc.get_queue(10).is_none());
    assert!(proc.get_queue(20).is_none());

    // 2. Nothing to process
    let remaining = tick(&mut proc, 20, 30);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "flow_scenario - 2");
    proc.assert_last_tick(30);

    // 3. New events are dispatched to future tick
    let mut new_expected_results = vec![];
    make_test_data_and_result!(event_source, new_expected_results, 5,
        50 => Group, Group, Roles, Roles, Rewards;
    );
    proc.update_event_source(vec![50], event_source);
    let remaining = tick(&mut proc, 20, 41);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "flow_scenario - 3");
    proc.assert_last_tick(40);
    assert!(proc.get_queue(50).is_some());

    // 4. Tick is too late
    expected_results.append(&mut new_expected_results);
    let remaining = tick(&mut proc, 20, 80);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "flow_scenario - 4");
    proc.assert_last_tick(80);
    assert!(proc.get_queue(50).is_none());

    // 5. Nothing to process
    let remaining = tick(&mut proc, 20, 90);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "flow_scenario - 5");
    proc.assert_last_tick(90);

    // 6. Nothing to process
    let remaining = tick(&mut proc, 20, 199);
    assert_eq!(remaining, 0);
    proc.assert_result(expected_results.clone(), "flow_scenario - 6");
    proc.assert_last_tick(190);
}
