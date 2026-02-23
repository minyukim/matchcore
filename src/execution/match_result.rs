use crate::{Side, execution::Trade};

use serde::{Deserialize, Serialize};

/// Result of a match operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// The side of the taker order
    taker_side: Side,
    /// The total executed quantity during the match
    executed_quantity: u64,
    /// The total value of the trades made during the match
    executed_value: u64,
    /// The trades that were made during the match
    trades: Vec<Trade>,
    /// The IDs of the orders that expired during the match
    expired_order_ids: Vec<u64>,
}

impl MatchResult {
    /// Create a new match result
    pub fn new(taker_side: Side) -> Self {
        Self {
            taker_side,
            executed_quantity: 0,
            executed_value: 0,
            trades: Vec::new(),
            expired_order_ids: Vec::new(),
        }
    }

    /// Get the side of the taker order
    pub fn taker_side(&self) -> Side {
        self.taker_side
    }

    /// Get the total executed quantity during the match
    pub fn executed_quantity(&self) -> u64 {
        self.executed_quantity
    }

    /// Get the total value of the trades made during the match
    pub fn executed_value(&self) -> u64 {
        self.executed_value
    }

    /// Get the trades that were made during the match
    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    /// Add a trade to the match result
    pub(crate) fn add_trade(&mut self, trade: Trade) {
        let price = trade.price();
        let quantity = trade.quantity();

        self.executed_quantity += quantity;
        self.executed_value += price * quantity;

        self.trades.push(trade);
    }

    /// Get the IDs of the orders that expired during the match
    pub fn expired_order_ids(&self) -> &[u64] {
        &self.expired_order_ids
    }

    /// Add an expired order ID to the match result
    pub(crate) fn add_expired_order_id(&mut self, order_id: u64) {
        self.expired_order_ids.push(order_id);
    }
}
