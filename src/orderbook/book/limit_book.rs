use crate::{LevelId, OrderId, Price, PriceLevel, RestingLimitOrder, Timestamp};

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap},
};

use rustc_hash::FxHashMap;
use slab::Slab;

/// Limit order book that manages limit orders and price levels
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct LimitBook<const LEVELS_INITIAL_CAPACITY: usize = 2048> {
    /// Limit orders indexed by order ID for O(1) lookup
    pub(crate) orders: FxHashMap<OrderId, RestingLimitOrder>,

    /// Price levels, stored in a slab with O(1) indexing
    pub(crate) levels: Slab<PriceLevel>,

    /// Bid side price levels, stored in a ordered map with O(log N) ordering
    pub(crate) bids: BTreeMap<Price, LevelId>,

    /// Ask side price levels, stored in a ordered map with O(log N) ordering
    pub(crate) asks: BTreeMap<Price, LevelId>,

    /// Queue of limit order IDs to be expired, stored in a min heap of tuples of
    /// (expires_at, order_id) with O(log N) push and pop
    pub(crate) expiration_queue: BinaryHeap<Reverse<(Timestamp, OrderId)>>,
}

impl<const LEVELS_INITIAL_CAPACITY: usize> LimitBook<LEVELS_INITIAL_CAPACITY> {
    /// Create a new limit order book
    pub fn new() -> Self {
        Self::default()
    }
}

impl LimitBook {
    /// Get the limit orders indexed by order ID
    pub fn orders(&self) -> &FxHashMap<OrderId, RestingLimitOrder> {
        &self.orders
    }

    /// Get the price levels
    pub fn levels(&self) -> &Slab<PriceLevel> {
        &self.levels
    }

    /// Get the bid side price levels
    pub fn bids(&self) -> &BTreeMap<Price, LevelId> {
        &self.bids
    }

    /// Get the ask side price levels
    pub fn asks(&self) -> &BTreeMap<Price, LevelId> {
        &self.asks
    }

    /// Get the queue of limit order IDs to be expired
    pub fn expiration_queue(&self) -> &BinaryHeap<Reverse<(Timestamp, OrderId)>> {
        &self.expiration_queue
    }

    /// Get the bid side price level for a given price
    pub fn get_bid_level(&self, price: Price) -> Option<&PriceLevel> {
        self.bids
            .get(&price)
            .map(|level_id| &self.levels[*level_id])
    }

    /// Get the ask side price level for a given price
    pub fn get_ask_level(&self, price: Price) -> Option<&PriceLevel> {
        self.asks
            .get(&price)
            .map(|level_id| &self.levels[*level_id])
    }
}
