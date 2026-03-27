use super::QueueEntry;

use std::collections::VecDeque;

/// Shared order tracking for a level
///
/// Stores the time-priority queue of order entries and the number of active orders.
/// Note that `order_count` may differ from `queue.len()` due to deferred cleanup of stale entries.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct LevelEntries {
    /// The number of orders at this level
    order_count: u64,
    /// The time priority queue of this level
    queue: VecDeque<QueueEntry>,
}

impl LevelEntries {
    /// Create a new level entries
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of orders at this level
    pub fn order_count(&self) -> u64 {
        self.order_count
    }

    /// Get the time priority queue of this price level
    pub fn queue(&self) -> &VecDeque<QueueEntry> {
        &self.queue
    }

    /// Increment the number of orders at this price level
    pub(crate) fn increment_order_count(&mut self) {
        self.order_count += 1;
    }

    /// Decrement the number of orders at this price level
    pub(crate) fn decrement_order_count(&mut self) {
        self.order_count -= 1;
    }

    /// Check if the price level is empty
    pub(crate) fn is_empty(&self) -> bool {
        self.order_count == 0
    }

    /// Push a queue entry to the queue
    pub(crate) fn push(&mut self, queue_entry: QueueEntry) {
        self.queue.push_back(queue_entry);
    }

    /// Attempt to peek the first queue entry in the queue without removing it
    pub(crate) fn peek(&self) -> Option<QueueEntry> {
        self.queue.front().copied()
    }

    /// Attempt to pop the first queue entry in the queue
    pub(crate) fn pop(&mut self) -> Option<QueueEntry> {
        self.queue.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_order_count() {
        let mut level_entries = LevelEntries::new();
        assert_eq!(level_entries.order_count(), 0);
        assert!(level_entries.is_empty());

        level_entries.increment_order_count();
        assert_eq!(level_entries.order_count(), 1);
        assert!(!level_entries.is_empty());

        level_entries.decrement_order_count();
        assert_eq!(level_entries.order_count(), 0);
        assert!(level_entries.is_empty());
    }

    #[test]
    fn test_push_peek_pop() {
        let mut level_entries = LevelEntries::new();
        assert!(level_entries.peek().is_none());

        level_entries.push(QueueEntry::new(SequenceNumber(0), OrderId(0)));
        assert_eq!(
            level_entries.peek(),
            Some(QueueEntry::new(SequenceNumber(0), OrderId(0)))
        );

        level_entries.push(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(
            level_entries.peek(),
            Some(QueueEntry::new(SequenceNumber(0), OrderId(0)))
        );

        let queue_entry = level_entries.pop();
        assert_eq!(
            queue_entry,
            Some(QueueEntry::new(SequenceNumber(0), OrderId(0)))
        );
        assert_eq!(
            level_entries.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );

        let queue_entry = level_entries.pop();
        assert_eq!(
            queue_entry,
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );
        assert!(level_entries.peek().is_none());
    }
}
