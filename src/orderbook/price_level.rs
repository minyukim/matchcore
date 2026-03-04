use crate::{LimitOrder, OrderId, Quantity};

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

/// Price level that manages the status of the orders with the same price.
/// It does not store the orders themselves, but only the queue of order IDs.
/// The orders are stored in the `OrderBook` struct for memory efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Total visible quantity at this price level
    visible_quantity: Quantity,
    /// Total hidden quantity at this price level
    hidden_quantity: Quantity,
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

    /// Consume the quantity at this price level
    pub(super) fn consume(&mut self, quantity: Quantity) {
        self.visible_quantity = self.visible_quantity.saturating_sub(quantity);
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
    fn push(&mut self, order_id: OrderId) {
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

    /// Push a limit order to the price level and add it to the order book
    pub(super) fn push_order(
        &mut self,
        limit_orders: &mut HashMap<OrderId, LimitOrder>,
        limit_order: LimitOrder,
    ) {
        self.visible_quantity += limit_order.visible_quantity();
        self.hidden_quantity += limit_order.hidden_quantity();

        self.push(limit_order.id());
        self.increment_order_count();
        limit_orders.insert(limit_order.id(), limit_order);
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

    /// Attempt to peek the first order in the price level without removing it
    /// It cleans up stale order IDs in the price level
    /// Returns a mutable reference to the order if it is found
    pub(super) fn peek_order<'a>(
        &mut self,
        limit_orders: &'a mut HashMap<OrderId, LimitOrder>,
    ) -> Option<&'a mut LimitOrder> {
        let order_id = self.peek_order_id(limit_orders)?;

        limit_orders.get_mut(&order_id)
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

    /// Handle the replenishment of the order
    /// If the replenishment quantity is 0, do nothing
    /// Otherwise, add the order back to the price level
    pub(super) fn handle_replenishment(&mut self, replenished_quantity: Quantity) {
        if replenished_quantity.is_zero() {
            return;
        }

        self.visible_quantity += replenished_quantity;
        self.hidden_quantity -= replenished_quantity;

        let Some(order_id) = self.pop() else {
            return;
        };
        self.push(order_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LimitOrder, LimitOrderSpec, OrderFlags, Price, Quantity, QuantityPolicy, Side, TimeInForce,
    };

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
    fn test_push_order() {
        let mut limit_orders = HashMap::new();
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.visible_quantity, Quantity(0));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 0);

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(0),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.visible_quantity, Quantity(10));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 1);

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(1),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(20),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.visible_quantity, Quantity(30));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 2);

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(2),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(10),
                        hidden_quantity: Quantity(20),
                        replenish_quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.visible_quantity, Quantity(40));
        assert_eq!(price_level.hidden_quantity, Quantity(20));
        assert_eq!(price_level.order_count(), 3);
    }

    #[test]
    fn test_peek_order_id() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(0),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(0)));

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(1),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(20),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(0)));
    }

    #[test]
    fn test_peek_order() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek_order(&mut limit_orders).is_none());

        let mut order = LimitOrder::new(
            OrderId(0),
            LimitOrderSpec::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        price_level.push_order(&mut limit_orders, order.clone());
        assert_eq!(price_level.peek_order(&mut limit_orders), Some(&mut order));

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(1),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(20),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.peek_order(&mut limit_orders), Some(&mut order));
    }

    #[test]
    fn test_remove_head_order() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(0),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(0)));

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(1),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(20),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(1)));

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(2),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(30),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(1)));

        price_level.remove_head_order(&mut limit_orders);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(2)));

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek_order_id(&limit_orders).is_none());
    }

    #[test]
    fn test_handle_replenishment() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.visible_quantity, Quantity(0));
        assert_eq!(price_level.hidden_quantity, Quantity(0));
        assert_eq!(price_level.order_count(), 0);
        assert!(price_level.peek_order(&mut limit_orders).is_none());

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(0),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(0),
                        hidden_quantity: Quantity(100),
                        replenish_quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.visible_quantity, Quantity(0));
        assert_eq!(price_level.hidden_quantity, Quantity(100));
        assert_eq!(price_level.order_count(), 1);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(0)));

        price_level.handle_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(10));
        assert_eq!(price_level.hidden_quantity, Quantity(90));
        assert_eq!(price_level.order_count(), 1);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(0)));

        price_level.push_order(
            &mut limit_orders,
            LimitOrder::new(
                OrderId(1),
                LimitOrderSpec::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(20),
                    },
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(price_level.visible_quantity, Quantity(30));
        assert_eq!(price_level.hidden_quantity, Quantity(90));
        assert_eq!(price_level.order_count(), 2);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(0)));

        price_level.handle_replenishment(Quantity(10));
        assert_eq!(price_level.visible_quantity, Quantity(40));
        assert_eq!(price_level.hidden_quantity, Quantity(80));
        assert_eq!(price_level.order_count(), 2);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(OrderId(1)));
    }
}
