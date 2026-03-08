use crate::{LimitOrder, OrderId, Quantity};

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

/// Price level that manages the status of the orders with the same price.
/// It does not store the orders themselves, but only the queue of order IDs.
/// The orders are stored in the `OrderBook` struct for memory efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Total visible quantity at this price level
    pub(super) visible_quantity: Quantity,
    /// Total hidden quantity at this price level
    pub(super) hidden_quantity: Quantity,
    /// Number of orders at this price level
    order_count: u64,
    /// Queue of order IDs at this price level
    order_ids: VecDeque<OrderId>,
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
            order_ids: VecDeque::new(),
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

    /// Increment the number of orders at this price level
    pub(super) fn increment_order_count(&mut self) {
        self.order_count += 1;
    }

    /// Decrement the number of orders at this price level
    pub(super) fn decrement_order_count(&mut self) {
        self.order_count -= 1;
    }

    /// Check if the price level is empty
    pub(super) fn is_empty(&self) -> bool {
        self.order_count == 0
    }
}

impl PriceLevel {
    /// Push an order ID to the queue
    pub(super) fn push(&mut self, order_id: OrderId) {
        self.order_ids.push_back(order_id);
    }

    /// Attempt to peek the first order ID in the queue without removing it
    fn peek(&self) -> Option<OrderId> {
        self.order_ids.front().copied()
    }

    /// Attempt to pop the first order ID in the queue
    fn pop(&mut self) -> Option<OrderId> {
        self.order_ids.pop_front()
    }

    /// Update the level when an order is added
    pub(super) fn on_order_added(&mut self, id: OrderId, visible: Quantity, hidden: Quantity) {
        self.visible_quantity += visible;
        self.hidden_quantity += hidden;

        self.push(id);
        self.increment_order_count();
    }

    /// Update the level when an order is removed
    /// Note that it does not remove the order ID from the queue.
    /// The stale order ID will be cleaned up when the order is peeked from the queue.
    pub(super) fn on_order_removed(&mut self, visible: Quantity, hidden: Quantity) {
        self.visible_quantity -= visible;
        self.hidden_quantity -= hidden;
        self.decrement_order_count();
    }

    /// Attempt to peek the first order ID in the price level without removing it
    /// It cleans up stale order IDs in the price level
    /// Returns the order ID if it is found
    pub(super) fn peek_order_id(
        &mut self,
        limit_orders: &HashMap<OrderId, LimitOrder>,
    ) -> Option<OrderId> {
        loop {
            let order_id = self.peek()?;
            if limit_orders.contains_key(&order_id) {
                return Some(order_id);
            }

            // Stale order ID in the price level, remove it
            self.pop();
        }
    }

    /// Pop the first order ID from the price level and remove it from the order book
    /// If the price level is empty, do nothing
    /// Note that it does not update the quantity of the price level
    pub(super) fn remove_head_order(&mut self, limit_orders: &mut HashMap<OrderId, LimitOrder>) {
        let Some(order_id) = self.pop() else {
            return;
        };
        limit_orders.remove(&order_id);
        self.decrement_order_count();
    }

    /// Handle the replenishment of an order in this price level
    /// It applies the replenished quantity and cycles the front order to the back.
    ///
    /// # Panics
    /// Panics if the queue is empty.
    pub(super) fn handle_replenishment(&mut self, replenished: Quantity) {
        self.apply_replenishment(replenished);
        self.cycle_front();
    }

    /// Apply the replenished quantity to the price level
    fn apply_replenishment(&mut self, replenished: Quantity) {
        self.visible_quantity += replenished;
        self.hidden_quantity -= replenished;
    }

    /// Rotates the queue by moving the front order ID to the back.
    ///
    /// This is used to preserve time priority when the front order
    /// remains active but should yield priority to other orders.
    ///
    /// # Panics
    /// Panics if the queue is empty.
    fn cycle_front(&mut self) {
        let order_id = self.pop().unwrap();
        self.push(order_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        price_level.on_order_added(OrderId(0), Quantity(10), Quantity(0));
        assert_eq!(price_level.visible_quantity, Quantity(10));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 1);

        price_level.on_order_added(OrderId(1), Quantity(20), Quantity(0));
        assert_eq!(price_level.visible_quantity, Quantity(30));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 2);

        price_level.on_order_added(OrderId(2), Quantity(30), Quantity(0));
        assert_eq!(price_level.visible_quantity, Quantity(60));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 3);

        price_level.on_order_added(OrderId(3), Quantity(40), Quantity(0));
        assert_eq!(price_level.visible_quantity, Quantity(100));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 4);

        price_level.on_order_added(OrderId(4), Quantity(50), Quantity(50));
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
    fn test_peek_order_id() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        // Push order 0
        limit_orders.insert(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        price_level.on_order_added(OrderId(0), Quantity(10), Quantity(0));
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(0)));

        // Push order 1, order 0 is still at the head of the queue
        limit_orders.insert(
            OrderId(1),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(20),
                },
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        price_level.on_order_added(OrderId(1), Quantity(20), Quantity(0));
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(0)));

        // Remove order 0, order 1 is now at the head of the queue
        limit_orders.remove(&OrderId(0));
        price_level.on_order_removed(Quantity(10), Quantity(0));
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(1)));

        // Remove order 1, the price level is empty
        limit_orders.remove(&OrderId(1));
        price_level.on_order_removed(Quantity(20), Quantity(0));
        assert!(price_level.peek_order_id(&limit_orders).is_none());
    }

    #[test]
    fn test_remove_head_order() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        limit_orders.insert(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        price_level.on_order_added(OrderId(0), Quantity(10), Quantity(0));
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(0)));

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        limit_orders.insert(
            OrderId(1),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(20),
                },
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        price_level.on_order_added(OrderId(1), Quantity(20), Quantity(0));
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(1)));

        limit_orders.insert(
            OrderId(2),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(30),
                },
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        price_level.on_order_added(OrderId(2), Quantity(30), Quantity(0));
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(1)));

        price_level.remove_head_order(&mut limit_orders);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(2)));

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek_order_id(&limit_orders).is_none());
    }

    #[test]
    fn test_cycle_front() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.peek(), None);

        price_level.push(OrderId(0));
        assert_eq!(price_level.peek(), Some(OrderId(0)));

        price_level.cycle_front();
        assert_eq!(price_level.peek(), Some(OrderId(0)));

        price_level.push(OrderId(1));
        assert_eq!(price_level.peek(), Some(OrderId(0)));

        price_level.cycle_front();
        assert_eq!(price_level.peek(), Some(OrderId(1)));

        price_level.cycle_front();
        assert_eq!(price_level.peek(), Some(OrderId(0)));
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
