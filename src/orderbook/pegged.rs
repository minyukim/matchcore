use super::PegLevel;
use crate::{OrderId, PegReference, PeggedOrder, Timestamp};

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use serde::{Deserialize, Serialize};

/// Order book that manages orders and levels.
/// It supports adding, updating, cancelling, and matching orders.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeggedBook {
    /// Pegged bid side levels, one for each reference price type
    pub(super) bid_levels: [PegLevel; PegReference::COUNT],

    /// Pegged ask side levels, one for each reference price type
    pub(super) ask_levels: [PegLevel; PegReference::COUNT],

    /// Pegged orders indexed by order ID for O(1) lookup
    pub(super) orders: HashMap<OrderId, PeggedOrder>,

    /// Queue of pegged order IDs to be expired, stored in a min heap of tuples of
    /// (expires_at, order_id) with O(log N) ordering
    pub(super) expiration_queue: BinaryHeap<Reverse<(Timestamp, OrderId)>>,
}

impl PeggedBook {
    /// Create a new pegged order book
    pub fn new() -> Self {
        Self::default()
    }
}
