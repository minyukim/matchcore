use super::PriceLevel;
use crate::{LimitOrder, OrderId, Price, Side, Timestamp};

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap, HashMap},
};

use serde::{Deserialize, Serialize};

/// Order book that manages orders and levels.
/// It supports adding, updating, cancelling, and matching orders.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LimitBook {
    /// Bid side price levels, stored in a ordered map with O(log N) ordering
    pub(super) bid_levels: BTreeMap<Price, PriceLevel>,

    /// Ask side price levels, stored in a ordered map with O(log N) ordering
    pub(super) ask_levels: BTreeMap<Price, PriceLevel>,

    /// Limit orders indexed by order ID for O(1) lookup
    pub(super) orders: HashMap<OrderId, LimitOrder>,

    /// Queue of limit order IDs to be expired, stored in a min heap of tuples of
    /// (expires_at, order_id) with O(log N) ordering
    pub(super) expiration_queue: BinaryHeap<Reverse<(Timestamp, OrderId)>>,
}

impl LimitBook {
    /// Create a new limit order book
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the bid side price levels
    pub fn bid_levels(&self) -> &BTreeMap<Price, PriceLevel> {
        &self.bid_levels
    }

    /// Get the ask side price levels
    pub fn ask_levels(&self) -> &BTreeMap<Price, PriceLevel> {
        &self.ask_levels
    }

    /// Get the limit orders indexed by order ID
    pub fn orders(&self) -> &HashMap<OrderId, LimitOrder> {
        &self.orders
    }

    /// Get the queue of limit order IDs to be expired
    pub fn expiration_queue(&self) -> &BinaryHeap<Reverse<(Timestamp, OrderId)>> {
        &self.expiration_queue
    }

    /// Get the best bid price, if any
    /// O(1) operation using the last key (highest price) in the BTreeMap
    pub fn best_bid(&self) -> Option<Price> {
        self.bid_levels.keys().next_back().copied()
    }

    /// Get the best ask price, if any
    /// O(1) operation using the first key (lowest price) in the BTreeMap
    pub fn best_ask(&self) -> Option<Price> {
        self.ask_levels.keys().next().copied()
    }

    /// Get the mid price (average of best bid and best ask)
    pub fn mid_price(&self) -> Option<f64> {
        let best_bid = self.best_bid()?;
        let best_ask = self.best_ask()?;
        Some((best_bid.as_f64() + best_ask.as_f64()) / 2.0)
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
            Side::Buy => self.bid_levels.is_empty(),
            Side::Sell => self.ask_levels.is_empty(),
        }
    }

    /// Check if there is a crossable order at the given limit price
    pub fn has_crossable_order(&self, taker_side: Side, limit_price: Price) -> bool {
        match taker_side {
            Side::Buy => self.best_ask().is_some_and(|ask| limit_price >= ask),
            Side::Sell => self.best_bid().is_some_and(|bid| limit_price <= bid),
        }
    }
}
