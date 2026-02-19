use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

/// Pegged order level that manages the status of the orders with the same pegged reference price.
/// Pegged orders do not have hidden quantity.
/// It does not store the orders themselves, but only the queue of order IDs.
/// The orders are stored in the `OrderBook` struct for memory efficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegLevel {
    /// Total quantity at this pegged order level
    pub quantity: u64,
    /// Number of orders at this pegged order level
    pub order_count: u64,
    /// Queue of order IDs at this pegged order level
    order_ids: VecDeque<u64>,
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
            quantity: 0,
            order_count: 0,
            order_ids: VecDeque::new(),
        }
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
