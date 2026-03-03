use crate::{OrderId, PegReference, PeggedOrder, Quantity};

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

/// Maker side array for primary peg reference price
pub(super) static MAKER_ARRAY_PRIMARY: [PegReference; 1] = [PegReference::Primary];
/// Maker side array for primary mid price peg reference price
pub(super) static MAKER_ARRAY_PRIMARY_MID_PRICE: [PegReference; 2] =
    [PegReference::Primary, PegReference::MidPrice];

/// Pegged order level that manages the status of the orders with the same pegged reference price.
/// Pegged orders do not have hidden quantity.
/// It does not store the orders themselves, but only the queue of order IDs.
/// The orders are stored in the `OrderBook` struct for memory efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegLevel {
    /// Total quantity at this pegged order level
    quantity: Quantity,
    /// Number of orders at this pegged order level
    order_count: u64,
    /// Queue of order IDs at this pegged order level
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

    /// Consume the quantity at this peg level
    pub(super) fn consume(&mut self, quantity: Quantity) {
        self.quantity = self.quantity.saturating_sub(quantity);
    }

    /// Get the number of orders at this peg level
    pub fn order_count(&self) -> u64 {
        self.order_count
    }

    /// Increment the number of orders at this peg level
    pub(super) fn increment_order_count(&mut self) {
        self.order_count += 1;
    }

    /// Decrement the number of orders at this peg level
    pub(super) fn decrement_order_count(&mut self) {
        self.order_count -= 1;
    }
}

impl PegLevel {
    /// Push an order ID to the queue
    fn _push(&mut self, order_id: OrderId) {
        self.order_ids.push_back(order_id);
    }

    /// Attempt to peek the first order ID in the queue without removing it
    fn _peek(&self) -> Option<OrderId> {
        self.order_ids.front().copied()
    }

    /// Attempt to pop the first order ID in the queue
    fn _pop(&mut self) -> Option<OrderId> {
        self.order_ids.pop_front()
    }

    /// Push a pegged order to the peg level and add it to the order book
    #[allow(unused)]
    pub(super) fn push(
        &mut self,
        pegged_orders: &mut HashMap<OrderId, PeggedOrder>,
        pegged_order: PeggedOrder,
    ) {
        self.quantity += pegged_order.quantity();

        self._push(pegged_order.id());
        self.increment_order_count();
        pegged_orders.insert(pegged_order.id(), pegged_order);
    }

    /// Attempt to peek the first order ID in the peg level without removing it
    /// It cleans up stale order IDs in the peg level
    /// Returns the order ID if it is found
    pub(super) fn peek_order_id(
        &mut self,
        pegged_orders: &HashMap<OrderId, PeggedOrder>,
    ) -> Option<OrderId> {
        loop {
            let order_id = self._peek()?;
            if pegged_orders.contains_key(&order_id) {
                return Some(order_id);
            }

            // Stale order ID in the price level, remove it
            self._pop();
        }
    }

    /// Attempt to peek the first order in the peg level without removing it
    /// It cleans up stale order IDs in the peg level
    /// Returns a mutable reference to the order if it is found
    #[allow(unused)]
    pub(super) fn peek<'a>(
        &mut self,
        pegged_orders: &'a mut HashMap<OrderId, PeggedOrder>,
    ) -> Option<&'a mut PeggedOrder> {
        let order_id = self.peek_order_id(pegged_orders)?;

        pegged_orders.get_mut(&order_id)
    }

    /// Pop the first order ID from the peg level and remove it from the order book
    /// If the peg level is empty, do nothing
    pub(super) fn remove_head_order(&mut self, pegged_orders: &mut HashMap<OrderId, PeggedOrder>) {
        let Some(order_id) = self._pop() else {
            return;
        };
        pegged_orders.remove(&order_id);
        self.decrement_order_count();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        OrderFlags, PegReference, PeggedOrder, PeggedOrderSpec, Quantity, Side, TimeInForce,
    };

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
    fn test_push() {
        let mut limit_orders = HashMap::new();
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.quantity, Quantity(0));
        assert_eq!(peg_level.order_count(), 0);

        peg_level.push(
            &mut limit_orders,
            PeggedOrder::new(
                OrderId(0),
                PeggedOrderSpec::new(
                    PegReference::Primary,
                    Quantity(10),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(peg_level.quantity, Quantity(10));
        assert_eq!(peg_level.order_count(), 1);

        peg_level.push(
            &mut limit_orders,
            PeggedOrder::new(
                OrderId(1),
                PeggedOrderSpec::new(
                    PegReference::Primary,
                    Quantity(20),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(peg_level.quantity, Quantity(30));
        assert_eq!(peg_level.order_count(), 2);

        peg_level.push(
            &mut limit_orders,
            PeggedOrder::new(
                OrderId(2),
                PeggedOrderSpec::new(
                    PegReference::Primary,
                    Quantity(30),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(peg_level.quantity, Quantity(60));
        assert_eq!(peg_level.order_count(), 3);
    }

    #[test]
    fn test_peek_order_id() {
        let mut pegged_orders = HashMap::new();

        let mut peg_level = PegLevel::new();
        assert!(peg_level.peek_order_id(&pegged_orders).is_none());

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderId(0),
                PeggedOrderSpec::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(OrderId(0)));

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderId(1),
                PeggedOrderSpec::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(OrderId(0)));
    }

    #[test]
    fn test_peek() {
        let mut pegged_orders = HashMap::new();

        let mut peg_level = PegLevel::new();
        assert!(peg_level.peek(&mut pegged_orders).is_none());

        let mut order = PeggedOrder::new(
            OrderId(0),
            PeggedOrderSpec::new(
                PegReference::Primary,
                Quantity(100),
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        );
        peg_level.push(&mut pegged_orders, order.clone());
        assert_eq!(peg_level.peek(&mut pegged_orders), Some(&mut order));

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderId(1),
                PeggedOrderSpec::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(peg_level.peek(&mut pegged_orders), Some(&mut order));
    }

    #[test]
    fn test_remove_head_order() {
        let mut pegged_orders = HashMap::new();

        let mut peg_level = PegLevel::new();
        assert!(peg_level.peek_order_id(&pegged_orders).is_none());

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderId(0),
                PeggedOrderSpec::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(OrderId(0)));

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek_order_id(&pegged_orders).is_none());

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderId(1),
                PeggedOrderSpec::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(OrderId(1)));

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderId(2),
                PeggedOrderSpec::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(OrderId(1)));

        peg_level.remove_head_order(&mut pegged_orders);
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(OrderId(2)));

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek_order_id(&pegged_orders).is_none());
    }
}
