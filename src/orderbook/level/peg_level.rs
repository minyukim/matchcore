use super::QueueEntry;
use crate::{OrderId, PegReference, Quantity, RestingPeggedOrder, SequenceNumber};

use std::collections::VecDeque;

use rustc_hash::FxHashMap;

/// Maker side array for the primary peg reference
pub(crate) static MAKER_ARRAY_PRIMARY: [PegReference; 1] = [PegReference::Primary];
/// Maker side array for the primary and mid price peg references
pub(crate) static MAKER_ARRAY_PRIMARY_MID_PRICE: [PegReference; 2] =
    [PegReference::Primary, PegReference::MidPrice];

/// Peg level that manages the status of the orders with the same peg reference.
/// It does not store the orders themselves, but only the time priority information of the orders.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct PegLevel {
    /// The sequence number at which the peg level was last repriced
    pub(crate) repriced_at: SequenceNumber,
    /// Total quantity at this peg level
    pub(crate) quantity: Quantity,
    /// Number of orders at this peg level
    order_count: u64,
    /// The time priority queue of this peg level
    queue: VecDeque<QueueEntry>,
}

impl Default for PegLevel {
    fn default() -> Self {
        Self::new()
    }
}

impl PegLevel {
    /// Create a new peg level
    pub fn new() -> Self {
        Self {
            repriced_at: SequenceNumber(0),
            quantity: Quantity(0),
            order_count: 0,
            queue: VecDeque::new(),
        }
    }

    /// Get the sequence number at which the peg level was last repriced
    pub fn repriced_at(&self) -> SequenceNumber {
        self.repriced_at
    }

    /// Get the quantity at this peg level
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Get the number of orders at this peg level
    pub fn order_count(&self) -> u64 {
        self.order_count
    }

    /// Get the time priority queue of this peg level
    pub fn queue(&self) -> &VecDeque<QueueEntry> {
        &self.queue
    }

    /// Increment the number of orders at this peg level
    pub(crate) fn increment_order_count(&mut self) {
        self.order_count += 1;
    }

    /// Decrement the number of orders at this peg level
    pub(crate) fn decrement_order_count(&mut self) {
        self.order_count -= 1;
    }
}

impl PegLevel {
    /// Push an order ID and time priority to the queue
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

    /// Add an order entry to the peg level
    pub(crate) fn add_order_entry(&mut self, queue_entry: QueueEntry, quantity: Quantity) {
        self.quantity += quantity;

        self.push(queue_entry);
        self.increment_order_count();
    }

    /// Mark an order as removed from the peg level
    /// Note that it does not remove the queue entry from the queue.
    /// The stale queue entry will be cleaned up when the order is peeked from the queue.
    pub(crate) fn mark_order_removed(&mut self, quantity: Quantity) {
        self.quantity -= quantity;
        self.decrement_order_count();
    }

    /// Pop the first queue entry from the peg level and remove the order from the order book
    /// If the peg level is empty, do nothing
    /// Note that it does not update the quantity of the peg level
    pub(crate) fn remove_head_order(
        &mut self,
        pegged_orders: &mut FxHashMap<OrderId, RestingPeggedOrder>,
    ) {
        let Some(queue_entry) = self.pop() else {
            return;
        };
        pegged_orders.remove(&queue_entry.order_id());
        self.decrement_order_count();
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{OrderFlags, PegReference, PeggedOrder, Quantity, Side, TimeInForce};

    use rustc_hash::FxHashMap;

    #[test]
    fn test_order_count() {
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.order_count(), 0);

        peg_level.increment_order_count();
        assert_eq!(peg_level.order_count(), 1);

        peg_level.decrement_order_count();
        assert_eq!(peg_level.order_count(), 0);
    }

    #[test]
    fn test_add_order_entry_and_mark_order_removed() {
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.quantity(), Quantity(0));
        assert_eq!(peg_level.order_count(), 0);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(0), OrderId(0)), Quantity(10));
        assert_eq!(peg_level.quantity(), Quantity(10));
        assert_eq!(peg_level.order_count(), 1);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)), Quantity(20));
        assert_eq!(peg_level.quantity(), Quantity(30));
        assert_eq!(peg_level.order_count(), 2);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(2), OrderId(2)), Quantity(30));
        assert_eq!(peg_level.quantity(), Quantity(60));
        assert_eq!(peg_level.order_count(), 3);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(3), OrderId(3)), Quantity(40));
        assert_eq!(peg_level.quantity(), Quantity(100));
        assert_eq!(peg_level.order_count(), 4);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(4), OrderId(4)), Quantity(50));
        assert_eq!(peg_level.quantity(), Quantity(150));
        assert_eq!(peg_level.order_count(), 5);

        peg_level.mark_order_removed(Quantity(10));
        assert_eq!(peg_level.quantity(), Quantity(140));
        assert_eq!(peg_level.order_count(), 4);

        peg_level.mark_order_removed(Quantity(20));
        assert_eq!(peg_level.quantity(), Quantity(120));
        assert_eq!(peg_level.order_count(), 3);

        peg_level.mark_order_removed(Quantity(30));
        assert_eq!(peg_level.quantity(), Quantity(90));
        assert_eq!(peg_level.order_count(), 2);

        peg_level.mark_order_removed(Quantity(40));
        assert_eq!(peg_level.quantity(), Quantity(50));
        assert_eq!(peg_level.order_count(), 1);

        peg_level.mark_order_removed(Quantity(50));
        assert_eq!(peg_level.quantity(), Quantity(0));
        assert_eq!(peg_level.order_count(), 0);
    }

    #[test]
    fn test_remove_head_order() {
        let mut pegged_orders = FxHashMap::default();

        let mut peg_level = PegLevel::new();
        assert!(peg_level.peek().is_none());

        pegged_orders.insert(
            OrderId(0),
            RestingPeggedOrder::new(
                SequenceNumber(0),
                PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        peg_level.add_order_entry(
            QueueEntry::new(SequenceNumber(0), OrderId(0)),
            Quantity(100),
        );
        assert_eq!(
            peg_level.peek(),
            Some(QueueEntry::new(SequenceNumber(0), OrderId(0)))
        );

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek().is_none());

        pegged_orders.insert(
            OrderId(1),
            RestingPeggedOrder::new(
                SequenceNumber(1),
                PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        peg_level.add_order_entry(
            QueueEntry::new(SequenceNumber(1), OrderId(1)),
            Quantity(100),
        );
        assert_eq!(
            peg_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );

        pegged_orders.insert(
            OrderId(2),
            RestingPeggedOrder::new(
                SequenceNumber(2),
                PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        peg_level.add_order_entry(
            QueueEntry::new(SequenceNumber(2), OrderId(2)),
            Quantity(100),
        );
        assert_eq!(
            peg_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );

        peg_level.remove_head_order(&mut pegged_orders);
        assert_eq!(
            peg_level.peek(),
            Some(QueueEntry::new(SequenceNumber(2), OrderId(2)))
        );

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek().is_none());
    }
}
