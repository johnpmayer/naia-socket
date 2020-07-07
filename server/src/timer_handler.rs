use futures_util::future::FutureExt;
use futures_util::stream::futures_unordered::FuturesUnordered;
use std::{collections::HashMap, time::Duration};
use tokio::time::{self, Interval};
use futures::prelude::*;
use tokio::time::Instant;

pub type TimerKey = u32;

#[derive(Debug)]
pub struct TimerHandler {
    current_timer_key: TimerKey,
    recycled_timer_keys: Vec<TimerKey>,
    timer_map: HashMap<TimerKey, Interval>,
}

impl TimerHandler {
    pub fn new() -> Self {
        TimerHandler {
            current_timer_key: 0,
            recycled_timer_keys: Vec::new(),
            timer_map: HashMap::new(),
        }
    }

    pub fn create_timer(&mut self, timer_interval: Duration) -> TimerKey {
        let new_timer = time::interval(timer_interval);
        let new_key = self.get_new_index();
        self.timer_map.insert(new_key, new_timer);
        return new_key;
    }

    pub fn delete_timer(&mut self, key: TimerKey) {
        if self.timer_map.contains_key(&key) {
            self.timer_map.remove(&key);
            self.recycled_timer_keys.push(key);
        }
    }

    pub fn get_futures(&self) -> FuturesUnordered<impl Future> {
        let mut futures = FuturesUnordered::<TimerKey>::new();
        for (timer_key, timer_interval) in self.timer_map.iter() {
            let future: bool = timer_interval.tick();
            futures.push(future);
        }
        return futures;
    }

    fn get_new_index(&mut self) -> TimerKey {
        if self.recycled_timer_keys.is_empty() {
            let new_index = self.current_timer_key;
            self.current_timer_key += 1;
            return new_index;
        }

        return self.recycled_timer_keys.pop().unwrap();
    }
}
