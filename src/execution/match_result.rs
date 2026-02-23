use crate::Side;

use std::fmt;

use serde::{Deserialize, Serialize};

/// Result of a match operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// The ID of the taker order
    taker_order_id: u64,
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
    pub fn new(taker_order_id: u64, taker_side: Side) -> Self {
        Self {
            taker_order_id,
            taker_side,
            executed_quantity: 0,
            executed_value: 0,
            trades: Vec::new(),
            expired_order_ids: Vec::new(),
        }
    }

    /// Get the ID of the taker order
    pub fn taker_order_id(&self) -> u64 {
        self.taker_order_id
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

    /// Get the IDs of the orders that expired during the match
    pub fn expired_order_ids(&self) -> &[u64] {
        &self.expired_order_ids
    }

    /// Add a trade to the match result
    pub fn add_trade(&mut self, trade: Trade) {
        let price = trade.price();
        let quantity = trade.quantity();

        self.executed_quantity += quantity;
        self.executed_value += price * quantity;

        self.trades.push(trade);
    }

    /// Add an expired order ID to the match result
    pub fn add_expired_order_id(&mut self, order_id: u64) {
        self.expired_order_ids.push(order_id);
    }
}

/// A trade that was made during a match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    /// The ID of the maker order
    maker_order_id: u64,
    /// The price of the trade
    price: u64,
    /// The quantity of the trade
    quantity: u64,
}

impl Trade {
    /// Create a new trade
    pub fn new(maker_order_id: u64, price: u64, quantity: u64) -> Self {
        Self {
            maker_order_id,
            price,
            quantity,
        }
    }

    /// Get the ID of the maker order
    pub fn maker_order_id(&self) -> u64 {
        self.maker_order_id
    }

    /// Get the price of the trade
    pub fn price(&self) -> u64 {
        self.price
    }

    /// Get the quantity of the trade
    pub fn quantity(&self) -> u64 {
        self.quantity
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Trade: maker_order_id={} price={} quantity={}",
            self.maker_order_id(),
            self.price(),
            self.quantity()
        )
    }
}
