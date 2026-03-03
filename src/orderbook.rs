mod amend;
mod cancel;
mod error;
mod execution;
mod matching;
mod operations;
mod peg_level;
mod price_level;
mod submit;
mod trigger;

pub use error::*;
pub use peg_level::*;
pub use price_level::*;

use crate::{LimitOrder, OrderId, PegReference, PeggedOrder, SequenceNumber, Side, Timestamp};

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

/// Order book that manages orders and levels.
/// It supports adding, updating, cancelling, and matching orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    /// The symbol for this order book
    symbol: String,

    /// The last sequence number of the order book, `None` if no command has been processed yet.
    /// This is used to ensure that the incoming commands are processed in the correct order.
    pub(self) last_sequence_number: Option<SequenceNumber>,

    /// The last seen timestamp of the order book, `None` if no command has been processed yet.
    /// This is used to ensure that the timestamps of the incoming commands are non-decreasing.
    pub(self) last_seen_timestamp: Option<Timestamp>,

    /// The last price at which a trade occurred, `None` if no trade has occurred yet
    pub(self) last_trade_price: Option<u64>,

    /// Bid side price levels, stored in a ordered map with O(log N) ordering
    pub(self) limit_bid_levels: BTreeMap<u64, PriceLevel>,

    /// Ask side price levels, stored in a ordered map with O(log N) ordering
    pub(self) limit_ask_levels: BTreeMap<u64, PriceLevel>,

    /// Limit orders indexed by order ID for O(1) lookup
    pub(self) limit_orders: HashMap<OrderId, LimitOrder>,

    /// Pegged bid side levels, one for each reference price type
    pub(self) peg_bid_levels: [PegLevel; PegReference::COUNT],

    /// Pegged ask side levels, one for each reference price type
    pub(self) peg_ask_levels: [PegLevel; PegReference::COUNT],

    /// Pegged orders indexed by order ID for O(1) lookup
    pub(self) pegged_orders: HashMap<OrderId, PeggedOrder>,
}

impl OrderBook {
    /// Create a new order book
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            last_sequence_number: None,
            last_seen_timestamp: None,
            last_trade_price: None,
            limit_bid_levels: BTreeMap::new(),
            limit_ask_levels: BTreeMap::new(),
            limit_orders: HashMap::new(),
            peg_bid_levels: core::array::from_fn(|_| PegLevel::new()),
            peg_ask_levels: core::array::from_fn(|_| PegLevel::new()),
            pegged_orders: HashMap::new(),
        }
    }

    /// Get the symbol for this order book
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Get the last sequence number of the order book, `None` if no command has been processed yet.
    pub fn last_sequence_number(&self) -> Option<SequenceNumber> {
        self.last_sequence_number
    }

    /// Get the last seen timestamp of the order book, `None` if no command has been processed yet.
    pub fn last_seen_timestamp(&self) -> Option<Timestamp> {
        self.last_seen_timestamp
    }

    /// Get the last trade price, `None` if no trade has occurred yet
    pub fn last_trade_price(&self) -> Option<u64> {
        self.last_trade_price
    }

    /// Get the best bid price, if any
    /// O(1) operation using the last key (highest price) in the BTreeMap
    pub fn best_bid(&self) -> Option<u64> {
        self.limit_bid_levels.keys().next_back().copied()
    }

    /// Get the best ask price, if any
    /// O(1) operation using the first key (lowest price) in the BTreeMap
    pub fn best_ask(&self) -> Option<u64> {
        self.limit_ask_levels.keys().next().copied()
    }

    /// Get the mid price (average of best bid and best ask)
    pub fn mid_price(&self) -> Option<f64> {
        let best_bid = self.best_bid()?;
        let best_ask = self.best_ask()?;
        Some((best_bid as f64 + best_ask as f64) / 2.0)
    }

    /// Get the spread (difference between best bid and best ask)
    pub fn spread(&self) -> Option<u64> {
        let best_bid = self.best_bid()?;
        let best_ask = self.best_ask()?;
        Some(best_ask - best_bid)
    }

    /// Check if the side is empty
    pub fn is_side_empty(&self, side: Side) -> bool {
        match side {
            Side::Buy => self.limit_bid_levels.is_empty(),
            Side::Sell => self.limit_ask_levels.is_empty(),
        }
    }

    /// Check if there is a crossable order at the given limit price
    pub fn has_crossable_order(&self, taker_side: Side, limit_price: u64) -> bool {
        match taker_side {
            Side::Buy => self.best_ask().is_some_and(|ask| limit_price >= ask),
            Side::Sell => self.best_bid().is_some_and(|bid| limit_price <= bid),
        }
    }
}
