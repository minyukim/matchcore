use crate::{
    Order, PeggedOrder,
    book::{PegLevel, PriceLevel},
};

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

const PEG_REFERENCE_COUNT: usize = 4;

/// Order book that manages orders and levels.
/// It supports adding, updating, cancelling, and matching orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook<E = ()>
where
    E: Clone + Copy + Eq + Serialize + core::fmt::Debug,
{
    /// The symbol for this order book
    symbol: String,

    /// The last price at which a trade occurred, `None` if no trade has occurred yet
    last_trade_price: Option<u64>,

    /// Bid side price levels, stored in a ordered map with O(log N) ordering
    bids: BTreeMap<u64, PriceLevel>,

    /// Ask side price levels, stored in a ordered map with O(log N) ordering
    asks: BTreeMap<u64, PriceLevel>,

    /// Orders indexed by order ID for O(1) lookup
    orders: HashMap<u64, Order<E>>,

    /// Pegged bid side levels, one for each reference price type
    pegged_bids: [PegLevel; PEG_REFERENCE_COUNT],

    /// Pegged ask side levels, one for each reference price type
    pegged_asks: [PegLevel; PEG_REFERENCE_COUNT],

    /// Pegged orders indexed by order ID for O(1) lookup
    pegged_orders: HashMap<u64, PeggedOrder<E>>,
}

impl<E: Clone + Copy + Eq + Serialize + core::fmt::Debug> OrderBook<E> {
    /// Create a new order book
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
            pegged_bids: core::array::from_fn(|_| PegLevel::new()),
            pegged_asks: core::array::from_fn(|_| PegLevel::new()),
            pegged_orders: HashMap::new(),
            last_trade_price: None,
        }
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
