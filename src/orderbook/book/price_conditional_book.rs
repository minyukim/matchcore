use crate::{OrderId, Price, RestingPriceConditionalOrder, TriggerPriceLevel};

use rustc_hash::FxHashMap;

/// Price-conditional order book that manages price-conditional orders
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct PriceConditionalBook {
    /// Price-conditional orders indexed by order ID for O(1) lookup
    pub(crate) orders: FxHashMap<OrderId, RestingPriceConditionalOrder>,

    /// Trigger price levels indexed by trigger price for O(1) lookup
    pub(crate) levels: FxHashMap<Price, TriggerPriceLevel>,
}

impl PriceConditionalBook {
    /// Create a new price-conditional order book
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the price-conditional orders indexed by order ID
    pub fn orders(&self) -> &FxHashMap<OrderId, RestingPriceConditionalOrder> {
        &self.orders
    }

    /// Get the trigger price levels indexed by trigger price
    pub fn levels(&self) -> &FxHashMap<Price, TriggerPriceLevel> {
        &self.levels
    }
}
