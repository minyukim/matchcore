use super::PriceLevel;
use crate::{LimitOrder, OrderId, Price, Timestamp};

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
    pub(crate) bid_levels: BTreeMap<Price, PriceLevel>,

    /// Ask side price levels, stored in a ordered map with O(log N) ordering
    pub(crate) ask_levels: BTreeMap<Price, PriceLevel>,

    /// Limit orders indexed by order ID for O(1) lookup
    pub(crate) orders: HashMap<OrderId, LimitOrder>,

    /// Queue of limit order IDs to be expired, stored in a min heap of tuples of
    /// (expires_at, order_id) with O(log N) ordering
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
    pub fn orders(&self) -> &HashMap<OrderId, LimitOrder> {
        &self.orders
    }

    /// Get the queue of limit order IDs to be expired
    pub fn expiration_queue(&self) -> &BinaryHeap<Reverse<(Timestamp, OrderId)>> {
        &self.expiration_queue
    }
}
