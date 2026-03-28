use crate::{LevelId, OrderId, Price, RestingPriceConditionalOrder, TriggerPriceLevel};

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;
use slab::Slab;

/// Price-conditional order book that manages price-conditional orders
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct PriceConditionalBook {
    /// Price-conditional orders indexed by order ID for O(1) lookup
    pub(crate) orders: FxHashMap<OrderId, RestingPriceConditionalOrder>,

    /// Trigger price levels, stored in a slab with O(1) indexing
    pub(crate) levels: Slab<TriggerPriceLevel>,

    /// Trigger prices, stored in a ordered map with O(log N) ordering
    pub(crate) trigger_prices: BTreeMap<Price, LevelId>,
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

    /// Get the trigger price levels
    pub fn levels(&self) -> &Slab<TriggerPriceLevel> {
        &self.levels
    }

    /// Get the trigger prices
    pub fn trigger_prices(&self) -> &BTreeMap<Price, LevelId> {
        &self.trigger_prices
    }

    /// Get the trigger price level for a given price
    pub fn get_level(&self, trigger_price: Price) -> Option<&TriggerPriceLevel> {
        self.trigger_prices
            .get(&trigger_price)
            .map(|level_id| &self.levels[*level_id])
    }
}
