use crate::{
    LimitOrder, PegReference, PeggedOrder,
    book::{PegLevel, PriceLevel},
};

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

/// Order book that manages orders and levels.
/// It supports adding, updating, cancelling, and matching orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook<E = ()>
where
    E: Clone + Copy + Eq + Serialize + core::fmt::Debug,
{
    /// The symbol for this order book
    symbol: String,

    /// The last sequence number of the order book
    /// This is used to ensure that the incoming orders are processed in the correct order.
    last_sequence_number: u64,

    /// The last processed timestamp of the order book, expressed as a Unix timestamp (seconds since epoch).
    /// This is used to ensure that the timestamps of the incoming orders are non-decreasing.
    last_processed_timestamp: u64,

    /// The last price at which a trade occurred, `None` if no trade has occurred yet
    last_trade_price: Option<u64>,

    /// Bid side price levels, stored in a ordered map with O(log N) ordering
    bids: BTreeMap<u64, PriceLevel>,

    /// Ask side price levels, stored in a ordered map with O(log N) ordering
    asks: BTreeMap<u64, PriceLevel>,

    /// Limit orders indexed by order ID for O(1) lookup
    limit_orders: HashMap<u64, LimitOrder<E>>,

    /// Pegged bid side levels, one for each reference price type
    pegged_bids: [PegLevel; PegReference::COUNT],

    /// Pegged ask side levels, one for each reference price type
    pegged_asks: [PegLevel; PegReference::COUNT],

    /// Pegged orders indexed by order ID for O(1) lookup
    pegged_orders: HashMap<u64, PeggedOrder<E>>,
}

impl<E: Clone + Copy + Eq + Serialize + core::fmt::Debug> OrderBook<E> {
    /// Create a new order book
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            last_sequence_number: 0,
            last_processed_timestamp: 0,
            last_trade_price: None,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            limit_orders: HashMap::new(),
            pegged_bids: core::array::from_fn(|_| PegLevel::new()),
            pegged_asks: core::array::from_fn(|_| PegLevel::new()),
            pegged_orders: HashMap::new(),
        }
    }

    /// Get the last sequence number of the order book
    pub fn last_sequence_number(&self) -> u64 {
        self.last_sequence_number
    }

    /// Get the last processed timestamp of the order book
    pub fn last_processed_timestamp(&self) -> u64 {
        self.last_processed_timestamp
    }

    pub fn last_trade_price(&self) -> Option<u64> {
        self.last_trade_price
    }

    /// Get the symbol for this order book
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Get the best bid price, if any
    /// O(1) operation using the last key (highest price) in the BTreeMap
    pub fn best_bid(&self) -> Option<u64> {
        self.bids.keys().next_back().copied()
    }

    /// Get the best ask price, if any
    /// O(1) operation using the first key (lowest price) in the BTreeMap
    pub fn best_ask(&self) -> Option<u64> {
        self.asks.keys().next().copied()
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
}
