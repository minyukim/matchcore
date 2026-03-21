use super::PriceLevel;
use crate::{OrderId, Price, RestingLimitOrder, Timestamp};

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap, HashMap},
};

/// Limit order book that manages limit orders and price levels.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct LimitBook {
    /// Bid side price levels, stored in a ordered map with O(log N) ordering
    pub(crate) bid_levels: BTreeMap<Price, PriceLevel>,

    /// Ask side price levels, stored in a ordered map with O(log N) ordering
    pub(crate) ask_levels: BTreeMap<Price, PriceLevel>,

    /// Limit orders indexed by order ID for O(1) lookup
    pub(crate) orders: HashMap<OrderId, RestingLimitOrder>,

    /// Queue of limit order IDs to be expired, stored in a min heap of tuples of
    /// (expires_at, order_id) with O(log N) push and pop
    pub(crate) expiration_queue: BinaryHeap<Reverse<(Timestamp, OrderId)>>,
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
    pub fn orders(&self) -> &HashMap<OrderId, RestingLimitOrder> {
        &self.orders
    }

    /// Get the queue of limit order IDs to be expired
    pub fn expiration_queue(&self) -> &BinaryHeap<Reverse<(Timestamp, OrderId)>> {
        &self.expiration_queue
    }
}
