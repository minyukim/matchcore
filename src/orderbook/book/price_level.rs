use super::QueueEntry;
use crate::{OrderId, Quantity, RestingLimitOrder, SequenceNumber};

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

/// Price level that manages the status of the orders with the same price.
/// It does not store the orders themselves, but only the time priority information of the orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Total visible quantity at this price level
    pub(crate) visible_quantity: Quantity,
    /// Total hidden quantity at this price level
    pub(crate) hidden_quantity: Quantity,
    /// Number of orders at this price level
    order_count: u64,
    /// The time priority queue of this price level
    queue: VecDeque<QueueEntry>,
}

impl Default for PriceLevel {
    fn default() -> Self {
        Self::new()
    }
}

impl PriceLevel {
    /// Create a new price level
    pub fn new() -> Self {
        Self {
            visible_quantity: Quantity(0),
            hidden_quantity: Quantity(0),
            order_count: 0,
            queue: VecDeque::new(),
        }
    }

    /// Get the visible quantity at this price level
    pub fn visible_quantity(&self) -> Quantity {
        self.visible_quantity
    }

    /// Get the hidden quantity at this price level
    pub fn hidden_quantity(&self) -> Quantity {
        self.hidden_quantity
    }

    /// Get the total quantity at this price level (visible + hidden)
    pub fn total_quantity(&self) -> Quantity {
        self.visible_quantity + self.hidden_quantity
    }

    /// Get the number of orders at this price level
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
}

impl PriceLevel {
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

    /// Update the level when an order is added
    pub(crate) fn on_order_added(
        &mut self,
        queue_entry: QueueEntry,
        visible: Quantity,
        hidden: Quantity,
    ) {
        self.visible_quantity += visible;
        self.hidden_quantity += hidden;

        self.push(queue_entry);
        self.increment_order_count();
    }

    /// Update the level when an order is removed
    /// Note that it does not remove the queue entry from the queue.
    /// The stale queue entry will be cleaned up when the order is peeked from the queue.
    pub(crate) fn on_order_removed(&mut self, visible: Quantity, hidden: Quantity) {
        self.visible_quantity -= visible;
        self.hidden_quantity -= hidden;
        self.decrement_order_count();
    }

    /// Pop the first queue entry from the price level and remove the order from the order book
    /// If the price level is empty, do nothing
    /// Note that it does not update the quantity of the price level
    pub(crate) fn remove_head_order(
        &mut self,
        limit_orders: &mut HashMap<OrderId, RestingLimitOrder>,
    ) {
        let Some(queue_entry) = self.pop() else {
            return;
        };
        limit_orders.remove(&queue_entry.order_id());
        self.decrement_order_count();
    }

    /// Apply the replenished quantity to the price level
    pub(crate) fn apply_replenishment(&mut self, replenished: Quantity) {
        self.visible_quantity += replenished;
        self.hidden_quantity -= replenished;
    }

    /// Reprioritize the front order and move it to the back of the queue
    ///
    /// # Panics
    /// Panics if the queue is empty.
    pub(crate) fn reprioritize_front(&mut self, time_priority: SequenceNumber) {
        let queue_entry = self.pop().unwrap();
        self.push(queue_entry.reprioritize(time_priority));
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{LimitOrder, OrderFlags, Price, Quantity, QuantityPolicy, Side, TimeInForce};

    use std::collections::HashMap;

    #[test]
    fn test_total_quantity() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.total_quantity(), Quantity(0));

        price_level.visible_quantity = Quantity(10);
        price_level.hidden_quantity = Quantity(20);
        assert_eq!(price_level.total_quantity(), Quantity(30));
    }

    #[test]
    fn test_order_count() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.order_count(), 0);
        assert!(price_level.is_empty());

        price_level.increment_order_count();
        assert_eq!(price_level.order_count(), 1);
        assert!(!price_level.is_empty());

        price_level.decrement_order_count();
        assert_eq!(price_level.order_count(), 0);
        assert!(price_level.is_empty());
    }

    #[test]
    fn test_on_order_added_and_removed() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.visible_quantity, Quantity(0));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 0);

        price_level.on_order_added(
            QueueEntry::new(SequenceNumber(0), OrderId(0)),
            Quantity(10),
            Quantity(0),
        );
        assert_eq!(price_level.visible_quantity, Quantity(10));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 1);

        price_level.on_order_added(
            QueueEntry::new(SequenceNumber(1), OrderId(1)),
            Quantity(20),
            Quantity(0),
        );
        assert_eq!(price_level.visible_quantity, Quantity(30));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 2);

        price_level.on_order_added(
            QueueEntry::new(SequenceNumber(2), OrderId(2)),
            Quantity(30),
            Quantity(0),
        );
        assert_eq!(price_level.visible_quantity, Quantity(60));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 3);

        price_level.on_order_added(
            QueueEntry::new(SequenceNumber(3), OrderId(3)),
            Quantity(40),
            Quantity(0),
        );
        assert_eq!(price_level.visible_quantity, Quantity(100));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 4);

        price_level.on_order_added(
            QueueEntry::new(SequenceNumber(4), OrderId(4)),
            Quantity(50),
            Quantity(50),
        );
        assert_eq!(price_level.visible_quantity, Quantity(150));
        assert_eq!(price_level.hidden_quantity, Quantity(50));
        assert_eq!(price_level.order_count(), 5);

        price_level.on_order_removed(Quantity(10), Quantity(0));
        assert_eq!(price_level.visible_quantity, Quantity(140));
        assert_eq!(price_level.hidden_quantity, Quantity(50));
        assert_eq!(price_level.order_count(), 4);

        price_level.on_order_removed(Quantity(20), Quantity(0));
        assert_eq!(price_level.visible_quantity, Quantity(120));
        assert_eq!(price_level.hidden_quantity, Quantity(50));
        assert_eq!(price_level.order_count(), 3);

        price_level.on_order_removed(Quantity(30), Quantity(0));
        assert_eq!(price_level.visible_quantity, Quantity(90));
        assert_eq!(price_level.hidden_quantity, Quantity(50));
        assert_eq!(price_level.order_count(), 2);

        price_level.on_order_removed(Quantity(40), Quantity(0));
        assert_eq!(price_level.visible_quantity, Quantity(50));
        assert_eq!(price_level.hidden_quantity, Quantity(50));
        assert_eq!(price_level.order_count(), 1);

        price_level.on_order_removed(Quantity(50), Quantity(50));
        assert_eq!(price_level.visible_quantity, Quantity(0));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 0);
    }

    #[test]
    fn test_remove_head_order() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek().is_none());

        limit_orders.insert(
            OrderId(0),
            RestingLimitOrder::new(
                SequenceNumber(0),
                LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        price_level.on_order_added(
            QueueEntry::new(SequenceNumber(0), OrderId(0)),
            Quantity(10),
            Quantity(0),
        );
        assert_eq!(
            price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(0), OrderId(0)))
        );

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek().is_none());

        limit_orders.insert(
            OrderId(1),
            RestingLimitOrder::new(
                SequenceNumber(1),
                LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(20),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        price_level.on_order_added(
            QueueEntry::new(SequenceNumber(1), OrderId(1)),
            Quantity(20),
            Quantity(0),
        );
        assert_eq!(
            price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );

        limit_orders.insert(
            OrderId(2),
            RestingLimitOrder::new(
                SequenceNumber(2),
                LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(30),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        price_level.on_order_added(
            QueueEntry::new(SequenceNumber(2), OrderId(2)),
            Quantity(30),
            Quantity(0),
        );
        assert_eq!(
            price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );

        price_level.remove_head_order(&mut limit_orders);
        assert_eq!(
            price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(2), OrderId(2)))
        );

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek().is_none());
    }

    #[test]
    fn test_reprioritize_front() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.peek(), None);

        price_level.push(QueueEntry::new(SequenceNumber(0), OrderId(0)));
        assert_eq!(
            price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(0), OrderId(0)))
        );

        price_level.reprioritize_front(SequenceNumber(1));
        assert_eq!(
            price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(0)))
        );

        price_level.push(QueueEntry::new(SequenceNumber(2), OrderId(2)));
        assert_eq!(
            price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(0)))
        );

        price_level.reprioritize_front(SequenceNumber(3));
        assert_eq!(
            price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(2), OrderId(2)))
        );

        price_level.reprioritize_front(SequenceNumber(4));
        assert_eq!(
            price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(3), OrderId(0)))
        );
    }

    #[test]
    fn test_apply_replenishment() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.visible_quantity, Quantity(0));
        assert_eq!(price_level.hidden_quantity, Quantity(0));

        price_level.visible_quantity = Quantity(10);
        price_level.hidden_quantity = Quantity(100);

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(20));
        assert_eq!(price_level.hidden_quantity, Quantity(90));

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(30));
        assert_eq!(price_level.hidden_quantity, Quantity(80));

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(40));
        assert_eq!(price_level.hidden_quantity, Quantity(70));

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(50));
        assert_eq!(price_level.hidden_quantity, Quantity(60));

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(60));
        assert_eq!(price_level.hidden_quantity, Quantity(50));

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(70));
        assert_eq!(price_level.hidden_quantity, Quantity(40));

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(80));
        assert_eq!(price_level.hidden_quantity, Quantity(30));

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(90));
        assert_eq!(price_level.hidden_quantity, Quantity(20));

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(100));
        assert_eq!(price_level.hidden_quantity, Quantity(10));

        price_level.apply_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(110));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
    }
}
