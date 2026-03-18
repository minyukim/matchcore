use crate::{OrderId, PegReference, PeggedOrder, Quantity};

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

/// Maker side array for the primary peg reference
pub(crate) static MAKER_ARRAY_PRIMARY: [PegReference; 1] = [PegReference::Primary];
/// Maker side array for the primary and mid price peg references
pub(crate) static MAKER_ARRAY_PRIMARY_MID_PRICE: [PegReference; 2] =
    [PegReference::Primary, PegReference::MidPrice];

/// Peg level that manages the status of the orders with the same peg reference.
/// It does not store the orders themselves, but only the queue of order IDs for the time-priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegLevel {
    /// Total quantity at this peg level
    pub(crate) quantity: Quantity,
    /// Number of orders at this peg level
    order_count: u64,
    /// Queue of order IDs at this peg level
    order_ids: VecDeque<OrderId>,
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
            quantity: Quantity(0),
            order_count: 0,
            order_ids: VecDeque::new(),
        }
    }

    /// Get the quantity at this peg level
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Get the number of orders at this peg level
    pub fn order_count(&self) -> u64 {
        self.order_count
    }

    /// Get the queue of order IDs at this peg level
    pub fn order_ids(&self) -> &VecDeque<OrderId> {
        &self.order_ids
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
    /// Push an order ID to the queue
    pub(crate) fn push(&mut self, order_id: OrderId) {
        self.order_ids.push_back(order_id);
    }

    /// Attempt to peek the first order ID in the queue without removing it
    pub(crate) fn peek(&self) -> Option<OrderId> {
        self.order_ids.front().copied()
    }

    /// Attempt to pop the first order ID in the queue
    pub(crate) fn pop(&mut self) -> Option<OrderId> {
        self.order_ids.pop_front()
    }

    /// Update the level when an order is added
    pub(crate) fn on_order_added(&mut self, id: OrderId, quantity: Quantity) {
        self.quantity += quantity;

        self.push(id);
        self.increment_order_count();
    }

    /// Update the level when an order is removed
    /// Note that it does not remove the order ID from the queue.
    /// The stale order ID will be cleaned up when the order is peeked from the queue.
    pub(crate) fn on_order_removed(&mut self, quantity: Quantity) {
        self.quantity -= quantity;
        self.decrement_order_count();
    }

    /// Pop the first order ID from the peg level and remove it from the order book
    /// If the peg level is empty, do nothing
    /// Note that it does not update the quantity of the peg level
    pub(crate) fn remove_head_order(&mut self, pegged_orders: &mut HashMap<OrderId, PeggedOrder>) {
        let Some(order_id) = self.pop() else {
            return;
        };
        pegged_orders.remove(&order_id);
        self.decrement_order_count();
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{OrderFlags, PegReference, PeggedOrder, Quantity, Side, TimeInForce};

    use std::collections::HashMap;

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
    fn test_on_order_added_and_removed() {
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.quantity(), Quantity(0));
        assert_eq!(peg_level.order_count(), 0);

        peg_level.on_order_added(OrderId(0), Quantity(10));
        assert_eq!(peg_level.quantity(), Quantity(10));
        assert_eq!(peg_level.order_count(), 1);

        peg_level.on_order_added(OrderId(1), Quantity(20));
        assert_eq!(peg_level.quantity(), Quantity(30));
        assert_eq!(peg_level.order_count(), 2);

        peg_level.on_order_added(OrderId(2), Quantity(30));
        assert_eq!(peg_level.quantity(), Quantity(60));
        assert_eq!(peg_level.order_count(), 3);

        peg_level.on_order_added(OrderId(3), Quantity(40));
        assert_eq!(peg_level.quantity(), Quantity(100));
        assert_eq!(peg_level.order_count(), 4);

        peg_level.on_order_added(OrderId(4), Quantity(50));
        assert_eq!(peg_level.quantity(), Quantity(150));
        assert_eq!(peg_level.order_count(), 5);

        peg_level.on_order_removed(Quantity(10));
        assert_eq!(peg_level.quantity(), Quantity(140));
        assert_eq!(peg_level.order_count(), 4);

        peg_level.on_order_removed(Quantity(20));
        assert_eq!(peg_level.quantity(), Quantity(120));
        assert_eq!(peg_level.order_count(), 3);

        peg_level.on_order_removed(Quantity(30));
        assert_eq!(peg_level.quantity(), Quantity(90));
        assert_eq!(peg_level.order_count(), 2);

        peg_level.on_order_removed(Quantity(40));
        assert_eq!(peg_level.quantity(), Quantity(50));
        assert_eq!(peg_level.order_count(), 1);

        peg_level.on_order_removed(Quantity(50));
        assert_eq!(peg_level.quantity(), Quantity(0));
        assert_eq!(peg_level.order_count(), 0);
    }

    #[test]
    fn test_remove_head_order() {
        let mut pegged_orders = HashMap::new();

        let mut peg_level = PegLevel::new();
        assert!(peg_level.peek().is_none());

        pegged_orders.insert(
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(100),
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        peg_level.on_order_added(OrderId(0), Quantity(100));
        assert_eq!(peg_level.peek(), Some(OrderId(0)));

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek().is_none());

        pegged_orders.insert(
            OrderId(1),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(100),
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        peg_level.on_order_added(OrderId(1), Quantity(100));
        assert_eq!(peg_level.peek(), Some(OrderId(1)));

        pegged_orders.insert(
            OrderId(2),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(100),
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        peg_level.on_order_added(OrderId(2), Quantity(100));
        assert_eq!(peg_level.peek(), Some(OrderId(1)));

        peg_level.remove_head_order(&mut pegged_orders);
        assert_eq!(peg_level.peek(), Some(OrderId(2)));

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek().is_none());
    }
}
