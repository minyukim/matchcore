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
    #[allow(unused)]
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

    /// Attempt to peek the first order ID in the price level without removing it
    /// It cleans up stale order IDs in the price level
    /// Returns the order ID if it is found
    pub(super) fn peek_order_id<E>(
        &mut self,
        limit_orders: &HashMap<u64, LimitOrder<E>>,
    ) -> Option<u64>
    where
        E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
    {
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
    pub(super) fn peek<'a, E>(
        &mut self,
        limit_orders: &'a mut HashMap<u64, LimitOrder<E>>,
    ) -> Option<&'a mut LimitOrder<E>>
    where
        E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
    {
        let order_id = self.peek_order_id(limit_orders)?;

        limit_orders.get_mut(&order_id)
    }

    /// Pop the first order ID from the price level and remove it from the order book
    /// If the price level is empty, do nothing
    pub(super) fn remove_head_order<E>(&mut self, limit_orders: &mut HashMap<u64, LimitOrder<E>>)
    where
        E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
    {
        let Some(order_id) = self._pop() else {
            return;
        };
        limit_orders.remove(&order_id);
        self.decrement_order_count();
    }
}
