use std::collections::VecDeque;

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
    pub order_count: u64,
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

    /// Push an order ID to the queue
    pub fn push(&mut self, order_id: u64) {
        self.order_ids.push_back(order_id);
    }

    /// Attempt to peek the first order ID in the queue without removing it
    pub fn peek(&self) -> Option<u64> {
        self.order_ids.front().copied()
    }

    /// Attempt to pop the first order ID in the queue
    pub fn pop(&mut self) -> Option<u64> {
        self.order_ids.pop_front()
    }
}
