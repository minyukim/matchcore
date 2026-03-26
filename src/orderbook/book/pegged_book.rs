use crate::{OrderId, PegLevel, PegReference, RestingPeggedOrder, Timestamp};

use std::{cmp::Reverse, collections::BinaryHeap};

use rustc_hash::FxHashMap;

/// Pegged order book that manages pegged orders and peg levels
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct PeggedBook {
    /// Pegged orders indexed by order ID for O(1) lookup
    pub(crate) orders: FxHashMap<OrderId, RestingPeggedOrder>,

    /// Pegged bid side levels, one for each reference price type
    pub(crate) bid_levels: [PegLevel; PegReference::COUNT],

    /// Pegged ask side levels, one for each reference price type
    pub(crate) ask_levels: [PegLevel; PegReference::COUNT],

    /// Queue of pegged order IDs to be expired, stored in a min heap of tuples of
    /// (expires_at, order_id) with O(log N) push and pop
    pub(crate) expiration_queue: BinaryHeap<Reverse<(Timestamp, OrderId)>>,
}

impl PeggedBook {
    /// Create a new pegged order book
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the pegged orders indexed by order ID
    pub fn orders(&self) -> &FxHashMap<OrderId, RestingPeggedOrder> {
        &self.orders
    }

    /// Get the pegged bid side levels
    pub fn bid_levels(&self) -> &[PegLevel; PegReference::COUNT] {
        &self.bid_levels
    }

    /// Get the pegged ask side levels
    pub fn ask_levels(&self) -> &[PegLevel; PegReference::COUNT] {
        &self.ask_levels
    }

    /// Get the queue of pegged order IDs to be expired
    pub fn expiration_queue(&self) -> &BinaryHeap<Reverse<(Timestamp, OrderId)>> {
        &self.expiration_queue
    }
}
