mod analytics;
mod book;
mod error;
mod execution;
mod logic;
mod market_data;

pub use analytics::*;
pub use book::*;
pub use error::*;
pub use market_data::*;

use crate::{Price, Quantity, SequenceNumber, Side, Timestamp};

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
    pub(self) last_trade_price: Option<Price>,

    /// Limit order book
    pub(self) limit: LimitBook,

    /// Pegged order book
    pub(self) pegged: PeggedBook,
}

impl OrderBook {
    /// Create a new order book
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            last_sequence_number: None,
            last_seen_timestamp: None,
            last_trade_price: None,
            limit: LimitBook::new(),
            pegged: PeggedBook::new(),
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
    pub fn last_trade_price(&self) -> Option<Price> {
        self.last_trade_price
    }

    /// Get the limit order book
    pub fn limit(&self) -> &LimitBook {
        &self.limit
    }

    /// Get the pegged order book
    pub fn pegged(&self) -> &PeggedBook {
        &self.pegged
    }

    /// Get the best bid price, if any
    pub fn best_bid_price(&self) -> Option<Price> {
        self.limit.best_bid_price()
    }

    /// Get the best ask price, if any
    pub fn best_ask_price(&self) -> Option<Price> {
        self.limit.best_ask_price()
    }

    /// Get the mid price (average of best bid and best ask)
    pub fn mid_price(&self) -> Option<f64> {
        self.limit.mid_price()
    }

    /// Get the spread (difference between best bid and best ask)
    pub fn spread(&self) -> Option<u64> {
        self.limit.spread()
    }

    /// Get the best bid volume, if not empty
    pub fn best_bid_volume(&self) -> Option<Quantity> {
        self.limit.best_bid_volume()
    }

    /// Get the best ask volume, if not empty
    pub fn best_ask_volume(&self) -> Option<Quantity> {
        self.limit.best_ask_volume()
    }

    /// Check if the side is empty
    pub fn is_side_empty(&self, side: Side) -> bool {
        self.limit.is_side_empty(side)
    }

    /// Check if there is a crossable order at the given limit price
    pub fn has_crossable_order(&self, taker_side: Side, limit_price: Price) -> bool {
        self.limit.has_crossable_order(taker_side, limit_price)
    }
}
