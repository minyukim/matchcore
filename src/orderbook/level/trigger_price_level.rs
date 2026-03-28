#![allow(dead_code)]

use super::{LevelEntries, QueueEntry};
use crate::{OrderId, RestingPriceConditionalOrder};

use std::ops::{Deref, DerefMut};

use rustc_hash::FxHashMap;

/// Trigger price level that manages the status of the orders with the same trigger price.
/// It does not store the orders themselves, but only the time priority information of the orders.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct TriggerPriceLevel {
    /// The level entries for this trigger price level
    level_entries: LevelEntries,
}

impl Default for TriggerPriceLevel {
    fn default() -> Self {
        Self::new()
    }
}

impl TriggerPriceLevel {
    /// Create a new trigger price level
    pub fn new() -> Self {
        Self {
            level_entries: LevelEntries::new(),
        }
    }

    /// Add an order entry to the trigger price level
    pub(crate) fn add_order_entry(&mut self, queue_entry: QueueEntry) {
        self.push(queue_entry);
        self.increment_order_count();
    }

    /// Mark an order as removed from the trigger price level
    /// Note that it does not remove the queue entry from the queue.
    /// The stale queue entry will be cleaned up when the order is peeked from the queue.
    pub(crate) fn mark_order_removed(&mut self) {
        self.decrement_order_count();
    }

    /// Pop the first queue entry from the trigger price level and remove the order from the order book
    /// If the trigger price level is empty, do nothing
    pub(crate) fn remove_head_order(
        &mut self,
        conditional_orders: &mut FxHashMap<OrderId, RestingPriceConditionalOrder>,
    ) {
        let Some(queue_entry) = self.pop() else {
            return;
        };
        conditional_orders.remove(&queue_entry.order_id());
        self.decrement_order_count();
    }
}

impl Deref for TriggerPriceLevel {
    type Target = LevelEntries;

    fn deref(&self) -> &Self::Target {
        &self.level_entries
    }
}
impl DerefMut for TriggerPriceLevel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.level_entries
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    use rustc_hash::FxHashMap;

    #[test]
    fn test_order_count() {
        let mut trigger_price_level = TriggerPriceLevel::new();
        assert_eq!(trigger_price_level.order_count(), 0);
        assert!(trigger_price_level.is_empty());

        trigger_price_level.increment_order_count();
        assert_eq!(trigger_price_level.order_count(), 1);
        assert!(!trigger_price_level.is_empty());

        trigger_price_level.decrement_order_count();
        assert_eq!(trigger_price_level.order_count(), 0);
        assert!(trigger_price_level.is_empty());
    }

    #[test]
    fn test_add_order_entry_and_mark_order_removed() {
        let mut trigger_price_level = TriggerPriceLevel::new();
        assert_eq!(trigger_price_level.order_count(), 0);

        trigger_price_level.add_order_entry(QueueEntry::new(SequenceNumber(0), OrderId(0)));
        assert_eq!(trigger_price_level.order_count(), 1);

        trigger_price_level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(trigger_price_level.order_count(), 2);

        trigger_price_level.add_order_entry(QueueEntry::new(SequenceNumber(2), OrderId(2)));
        assert_eq!(trigger_price_level.order_count(), 3);

        trigger_price_level.add_order_entry(QueueEntry::new(SequenceNumber(3), OrderId(3)));
        assert_eq!(trigger_price_level.order_count(), 4);

        trigger_price_level.add_order_entry(QueueEntry::new(SequenceNumber(4), OrderId(4)));
        assert_eq!(trigger_price_level.order_count(), 5);

        trigger_price_level.mark_order_removed();
        assert_eq!(trigger_price_level.order_count(), 4);

        trigger_price_level.mark_order_removed();
        assert_eq!(trigger_price_level.order_count(), 3);

        trigger_price_level.mark_order_removed();
        assert_eq!(trigger_price_level.order_count(), 2);

        trigger_price_level.mark_order_removed();
        assert_eq!(trigger_price_level.order_count(), 1);

        trigger_price_level.mark_order_removed();
        assert_eq!(trigger_price_level.order_count(), 0);
    }

    #[test]
    fn test_remove_head_order() {
        let mut conditional_orders = FxHashMap::default();

        let mut trigger_price_level = TriggerPriceLevel::new();
        assert!(trigger_price_level.peek().is_none());

        conditional_orders.insert(
            OrderId(0),
            RestingPriceConditionalOrder::new(
                SequenceNumber(0),
                PriceConditionalOrder::new(
                    Price(100),
                    TriggerDirection::AtOrAbove,
                    TriggerOrder::Market(MarketOrder::new(Quantity(10), Side::Buy, false)),
                ),
            ),
        );
        trigger_price_level.add_order_entry(QueueEntry::new(SequenceNumber(0), OrderId(0)));
        assert_eq!(
            trigger_price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(0), OrderId(0)))
        );

        trigger_price_level.remove_head_order(&mut conditional_orders);
        assert!(trigger_price_level.peek().is_none());

        conditional_orders.insert(
            OrderId(1),
            RestingPriceConditionalOrder::new(
                SequenceNumber(1),
                PriceConditionalOrder::new(
                    Price(100),
                    TriggerDirection::AtOrAbove,
                    TriggerOrder::Market(MarketOrder::new(Quantity(20), Side::Buy, false)),
                ),
            ),
        );
        trigger_price_level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(
            trigger_price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );

        conditional_orders.insert(
            OrderId(2),
            RestingPriceConditionalOrder::new(
                SequenceNumber(2),
                PriceConditionalOrder::new(
                    Price(100),
                    TriggerDirection::AtOrAbove,
                    TriggerOrder::Market(MarketOrder::new(Quantity(30), Side::Buy, false)),
                ),
            ),
        );
        trigger_price_level.add_order_entry(QueueEntry::new(SequenceNumber(2), OrderId(2)));
        assert_eq!(
            trigger_price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );

        trigger_price_level.remove_head_order(&mut conditional_orders);
        assert_eq!(
            trigger_price_level.peek(),
            Some(QueueEntry::new(SequenceNumber(2), OrderId(2)))
        );

        trigger_price_level.remove_head_order(&mut conditional_orders);
        assert!(trigger_price_level.peek().is_none());
    }
}
