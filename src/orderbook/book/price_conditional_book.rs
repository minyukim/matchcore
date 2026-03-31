use crate::{
    LevelId, OrderId, Price, PriceConditionalOrder, RestingPriceConditionalOrder, TriggerPriceLevel,
};

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

    /// The entries added before the first trade occurs
    pub(crate) pre_trade_level: TriggerPriceLevel,
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

    /// Get the entries added before the first trade occurs
    pub fn pre_trade_level(&self) -> &TriggerPriceLevel {
        &self.pre_trade_level
    }

    /// Get the trigger price level for a given price
    pub fn get_level(&self, trigger_price: Price) -> Option<&TriggerPriceLevel> {
        self.trigger_prices
            .get(&trigger_price)
            .map(|level_id| &self.levels[*level_id])
    }

    /// Removes and returns all the orders in the levels strictly after `start_exclusive`
    /// and up to and including `end_inclusive` from the order book.
    #[allow(dead_code)]
    pub(crate) fn drain_levels(
        &mut self,
        start_exclusive: Price,
        end_inclusive: Price,
    ) -> Vec<(OrderId, PriceConditionalOrder)> {
        if start_exclusive == end_inclusive {
            return Vec::new();
        }

        let reverse = start_exclusive > end_inclusive;
        let lower_inclusive = if reverse {
            end_inclusive
        } else {
            start_exclusive.inc()
        };
        let upper_exclusive = if reverse {
            start_exclusive
        } else {
            end_inclusive.inc()
        };

        let mut middle = self.trigger_prices.split_off(&lower_inclusive);
        let mut tail = middle.split_off(&upper_exclusive);
        self.trigger_prices.append(&mut tail);

        let level_ids: Vec<_> = if reverse {
            middle.values().rev().copied().collect()
        } else {
            middle.values().copied().collect()
        };

        let levels: Vec<_> = level_ids
            .into_iter()
            .map(|level_id| self.levels.remove(level_id))
            .collect();

        let mut orders = Vec::new();
        for mut level in levels {
            orders.extend(level.drain_orders(&mut self.orders));
        }

        orders
    }

    /// Drains all orders from the pre-trade level
    #[allow(dead_code)]
    pub(crate) fn drain_pre_trade_level(&mut self) -> Vec<(OrderId, PriceConditionalOrder)> {
        let orders = self.pre_trade_level.drain_orders(&mut self.orders);
        self.pre_trade_level = TriggerPriceLevel::new(); // Deallocate the level
        orders
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::collections::BTreeMap;

    fn insert_level_with_single_order(
        book: &mut PriceConditionalBook,
        trigger_price: Price,
        time_priority: SequenceNumber,
    ) {
        let order_id = OrderId::from(time_priority);

        book.add_order(
            time_priority,
            order_id,
            PriceConditionalOrder::new(
                trigger_price,
                TriggerDirection::AtOrAbove,
                TriggerOrder::Market(MarketOrder::new(Quantity(1), Side::Buy, false)),
            ),
        );
    }

    fn create_book(prices: &[u64]) -> PriceConditionalBook {
        let mut book = PriceConditionalBook::new();
        for (i, p) in prices.iter().copied().enumerate() {
            insert_level_with_single_order(&mut book, Price(p), SequenceNumber(i as u64));
        }
        book
    }

    #[test]
    fn drain_levels_start_equals_end_returns_empty_and_no_changes() {
        let prices = [100, 101, 102];
        let mut book = create_book(&prices);

        let before_trigger_prices = book.trigger_prices.clone();
        let before_orders_len = book.orders.len();

        let drained = book.drain_levels(Price(101), Price(101));
        assert!(drained.is_empty());
        assert_eq!(book.trigger_prices, before_trigger_prices);
        assert_eq!(book.orders.len(), before_orders_len);
    }

    #[test]
    fn drain_levels_forward_is_exclusive_inclusive_and_removes_state() {
        let prices = [100, 101, 102, 103, 104, 105, 106, 107, 108, 109];
        let mut book = create_book(&prices);

        let drained = book.drain_levels(Price(100), Price(105));
        let drained_prices: Vec<_> = drained
            .into_iter()
            .map(|(_, order)| order.trigger_price())
            .collect();
        assert_eq!(
            drained_prices,
            vec![Price(101), Price(102), Price(103), Price(104), Price(105)]
        );

        assert_eq!(
            book.trigger_prices.keys().copied().collect::<Vec<Price>>(),
            vec![Price(100), Price(106), Price(107), Price(108), Price(109)]
        );
        assert_eq!(book.orders.len(), 5);
        assert!(book.get_level(Price(101)).is_none());
        assert!(book.get_level(Price(105)).is_none());
        assert!(book.get_level(Price(106)).is_some());
    }

    #[test]
    fn drain_levels_reverse_drains_downward_including_end_inclusive() {
        let prices = [100, 101, 102, 103, 104, 105, 106];
        let mut book = create_book(&prices);

        let drained = book.drain_levels(Price(105), Price(100));
        let drained_prices: Vec<_> = drained
            .into_iter()
            .map(|(_, order)| order.trigger_price())
            .collect();
        assert_eq!(
            drained_prices,
            vec![Price(104), Price(103), Price(102), Price(101), Price(100)]
        );

        assert_eq!(
            book.trigger_prices,
            BTreeMap::from([
                (Price(105), *book.trigger_prices.get(&Price(105)).unwrap()),
                (Price(106), *book.trigger_prices.get(&Price(106)).unwrap())
            ])
        );
        assert_eq!(book.orders.len(), 2);
    }
}
