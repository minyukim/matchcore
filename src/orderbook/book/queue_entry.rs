use crate::{OrderId, SequenceNumber};

use serde::{Deserialize, Serialize};

/// Represents a time priority queue entry
///
/// Time priority rules:
/// 1. The order with the smaller sequence number is higher priority
/// 2. If the sequence numbers are the same, limit orders are higher priority than pegged orders
/// 3. If the pegged orders have the same sequence number, the order with the smaller order ID is higher priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QueueEntry {
    /// The time priority of the order
    time_priority: SequenceNumber,
    /// The order ID
    order_id: OrderId,
}

impl QueueEntry {
    /// Create a new queue entry
    pub fn new(time_priority: SequenceNumber, order_id: OrderId) -> Self {
        Self {
            time_priority,
            order_id,
        }
    }

    /// Get the time priority of the order
    pub fn time_priority(&self) -> SequenceNumber {
        self.time_priority
    }

    /// Reprioritize the order
    pub(crate) fn reprioritize(self, time_priority: SequenceNumber) -> Self {
        Self {
            time_priority,
            order_id: self.order_id,
        }
    }

    /// Get the order ID
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_priority() {
        let queue_entry = QueueEntry::new(SequenceNumber(0), OrderId(0));
        assert_eq!(queue_entry.time_priority(), SequenceNumber(0));

        let queue_entry = queue_entry.reprioritize(SequenceNumber(1));
        assert_eq!(queue_entry.time_priority(), SequenceNumber(1));
    }

    #[test]
    fn test_order_id() {
        let queue_entry = QueueEntry::new(SequenceNumber(0), OrderId(0));
        assert_eq!(queue_entry.order_id(), OrderId(0));
    }

    #[test]
    fn test_ord() {
        let queue_entry1 = QueueEntry::new(SequenceNumber(0), OrderId(0));
        let queue_entry2 = QueueEntry::new(SequenceNumber(0), OrderId(1));
        let queue_entry3 = QueueEntry::new(SequenceNumber(1), OrderId(0));
        let queue_entry4 = QueueEntry::new(SequenceNumber(1), OrderId(1));

        assert!(queue_entry1 < queue_entry2);
        assert!(queue_entry1 < queue_entry3);
        assert!(queue_entry1 < queue_entry4);
        assert!(queue_entry2 < queue_entry3);
        assert!(queue_entry2 < queue_entry4);
        assert!(queue_entry3 < queue_entry4);
    }
}
