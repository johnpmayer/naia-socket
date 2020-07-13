use std::{cmp::Ordering, collections::BinaryHeap};

use super::Instant;

/// A queue for items marked by time, will only ever pop items from the queue if
/// the time
#[derive(Debug)]
pub struct TimeQueue<T: Eq + PartialEq> {
    queue: BinaryHeap<ItemContainer<T>>,
}

impl<T: Eq + PartialEq> TimeQueue<T> {
    /// Create a new TimeQueue
    pub fn new() -> Self {
        TimeQueue {
            queue: BinaryHeap::new(),
        }
    }

    /// Adds an item to the queue marked by time
    pub fn add_item(&mut self, instant: Instant, item: T) {
        self.queue.push(ItemContainer { instant, item });
    }

    /// Returns whether or not there is an item that is ready to be returned
    pub fn has_item(&self) -> bool {
        if self.queue.len() == 0 {
            return false;
        }
        if let Some(item) = self.queue.peek() {
            return item.instant <= Instant::now();
        }
        return false;
    }

    /// Pops an item from the queue if the sufficient time has elapsed
    pub fn pop_item(&mut self) -> Option<T> {
        if self.has_item() {
            if let Some(container) = self.queue.pop() {
                return Some(container.item);
            }
        }
        return None;
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ItemContainer<T: Eq + PartialEq> {
    pub instant: Instant,
    pub item: T,
}

impl<T: Eq + PartialEq> Ord for ItemContainer<T> {
    fn cmp(&self, other: &ItemContainer<T>) -> Ordering {
        other.instant.cmp(&self.instant)
    }
}

impl<T: Eq + PartialEq> PartialOrd for ItemContainer<T> {
    fn partial_cmp(&self, other: &ItemContainer<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
