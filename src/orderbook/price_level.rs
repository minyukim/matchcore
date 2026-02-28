use crate::LimitOrder;

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

/// Price level that manages the status of the orders with the same price.
/// It does not store the orders themselves, but only the queue of order IDs.
/// The orders are stored in the `OrderBook` struct for memory efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Total visible quantity at this price level
    pub visible_quantity: u64,
    /// Total hidden quantity at this price level
    pub hidden_quantity: u64,
    /// Number of orders at this price level
    order_count: u64,
    /// Queue of order IDs at this price level
    order_ids: VecDeque<u64>,
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
            visible_quantity: 0,
            hidden_quantity: 0,
            order_count: 0,
            order_ids: VecDeque::new(),
        }
    }

    /// Get the total quantity at this price level (visible + hidden)
    pub fn total_quantity(&self) -> u64 {
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
    fn _push(&mut self, order_id: u64) {
        self.order_ids.push_back(order_id);
    }

    /// Attempt to peek the first order ID in the queue without removing it
    fn _peek(&self) -> Option<u64> {
        self.order_ids.front().copied()
    }

    /// Attempt to pop the first order ID in the queue
    fn _pop(&mut self) -> Option<u64> {
        self.order_ids.pop_front()
    }

    /// Push a limit order to the price level and add it to the order book
    pub(super) fn push(
        &mut self,
        limit_orders: &mut HashMap<u64, LimitOrder>,
        limit_order: LimitOrder,
    ) {
        self.visible_quantity += limit_order.visible_quantity();
        self.hidden_quantity += limit_order.hidden_quantity();

        self._push(limit_order.id());
        self.increment_order_count();
        limit_orders.insert(limit_order.id(), limit_order);
    }

    /// Attempt to peek the first order ID in the price level without removing it
    /// It cleans up stale order IDs in the price level
    /// Returns the order ID if it is found
    pub(super) fn peek_order_id(&mut self, limit_orders: &HashMap<u64, LimitOrder>) -> Option<u64> {
        loop {
            let order_id = self._peek()?;
            if limit_orders.contains_key(&order_id) {
                return Some(order_id);
            }

            // Stale order ID in the price level, remove it
            self._pop();
        }
    }

    /// Attempt to peek the first order in the price level without removing it
    /// It cleans up stale order IDs in the price level
    /// Returns a mutable reference to the order if it is found
    pub(super) fn peek<'a>(
        &mut self,
        limit_orders: &'a mut HashMap<u64, LimitOrder>,
    ) -> Option<&'a mut LimitOrder> {
        let order_id = self.peek_order_id(limit_orders)?;

        limit_orders.get_mut(&order_id)
    }

    /// Pop the first order ID from the price level and remove it from the order book
    /// If the price level is empty, do nothing
    pub(super) fn remove_head_order(&mut self, limit_orders: &mut HashMap<u64, LimitOrder>) {
        let Some(order_id) = self._pop() else {
            return;
        };
        limit_orders.remove(&order_id);
        self.decrement_order_count();
    }

    /// Handle the replenishment of the order
    /// If the replenishment quantity is 0, do nothing
    /// Otherwise, add the order back to the price level
    pub(super) fn handle_replenishment(&mut self, replenished_quantity: u64) {
        if replenished_quantity == 0 {
            return;
        }

        self.visible_quantity += replenished_quantity;
        self.hidden_quantity -= replenished_quantity;

        let Some(order_id) = self._pop() else {
            return;
        };
        self._push(order_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LimitOrder, OrderCore, QuantityPolicy, Side, TimeInForce};

    use std::collections::HashMap;

    #[test]
    fn test_total_quantity() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.total_quantity(), 0);

        price_level.visible_quantity = 10;
        price_level.hidden_quantity = 20;
        assert_eq!(price_level.total_quantity(), 30);
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
    fn test_push() {
        let mut limit_orders = HashMap::new();
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.visible_quantity, 0);
        assert_eq!(price_level.hidden_quantity, 0);
        assert_eq!(price_level.order_count(), 0);

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(0, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Standard { quantity: 10 },
            ),
        );
        assert_eq!(price_level.visible_quantity, 10);
        assert_eq!(price_level.hidden_quantity, 0);
        assert_eq!(price_level.order_count(), 1);

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.visible_quantity, 30);
        assert_eq!(price_level.hidden_quantity, 0);
        assert_eq!(price_level.order_count(), 2);

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(2, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Iceberg {
                    visible_quantity: 10,
                    hidden_quantity: 20,
                    replenish_quantity: 10,
                },
            ),
        );
        assert_eq!(price_level.visible_quantity, 40);
        assert_eq!(price_level.hidden_quantity, 20);
        assert_eq!(price_level.order_count(), 3);
    }

    #[test]
    fn test_peek_order_id() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(0, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Standard { quantity: 10 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));
    }

    #[test]
    fn test_peek() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek(&mut limit_orders).is_none());

        let mut order = LimitOrder::new(
            OrderCore::new(0, Side::Buy, true, TimeInForce::Gtc),
            100,
            QuantityPolicy::Standard { quantity: 10 },
        );
        price_level.push(&mut limit_orders, order.clone());
        assert_eq!(price_level.peek(&mut limit_orders), Some(&mut order));

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.peek(&mut limit_orders), Some(&mut order));
    }

    #[test]
    fn test_remove_head_order() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(0, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Standard { quantity: 10 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(1));

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(2, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Standard { quantity: 30 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(1));

        price_level.remove_head_order(&mut limit_orders);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(2));

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek_order_id(&limit_orders).is_none());
    }

    #[test]
    fn test_handle_replenishment() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.visible_quantity, 0);
        assert_eq!(price_level.hidden_quantity, 0);
        assert_eq!(price_level.order_count(), 0);
        assert!(price_level.peek(&mut limit_orders).is_none());

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(0, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Iceberg {
                    visible_quantity: 0,
                    hidden_quantity: 100,
                    replenish_quantity: 10,
                },
            ),
        );
        assert_eq!(price_level.visible_quantity, 0);
        assert_eq!(price_level.hidden_quantity, 100);
        assert_eq!(price_level.order_count(), 1);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.handle_replenishment(10);
        assert_eq!(price_level.visible_quantity, 10);
        assert_eq!(price_level.hidden_quantity, 90);
        assert_eq!(price_level.order_count(), 1);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, TimeInForce::Gtc),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.visible_quantity, 30);
        assert_eq!(price_level.hidden_quantity, 90);
        assert_eq!(price_level.order_count(), 2);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.handle_replenishment(10);
        assert_eq!(price_level.visible_quantity, 40);
        assert_eq!(price_level.hidden_quantity, 80);
        assert_eq!(price_level.order_count(), 2);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(1));
    }
}
