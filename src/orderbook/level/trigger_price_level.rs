use super::{LevelEntries, QueueEntry};
use crate::{OrderId, RestingPriceConditionalOrder};

use std::ops::{Deref, DerefMut};

use rustc_hash::FxHashMap;

/// Trigger price level that manages the status of the orders with the same trigger price.
/// It does not store the orders themselves, but only the time priority information of the orders.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct TriggerPriceLevel {
    /// The level entries for this trigger price level
    level_entries: LevelEntries,
}

impl TriggerPriceLevel {
    /// Create a new trigger price level
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the level entries for this trigger price level
    pub fn level_entries(&self) -> &LevelEntries {
        &self.level_entries
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

    /// Drains all valid orders from the level and the provided `orders` map
    pub(crate) fn drain_orders(
        &mut self,
        orders: &mut FxHashMap<OrderId, RestingPriceConditionalOrder>,
    ) -> Vec<RestingPriceConditionalOrder> {
        let mut orders_vec = Vec::with_capacity(self.order_count() as usize);

        while !self.is_empty() {
            let queue_entry = self.pop().unwrap();

            let Some(order) = orders.remove(&queue_entry.order_id()) else {
                continue; // Stale entry
            };
            if queue_entry.time_priority() != order.time_priority() {
                continue; // Stale entry
            }

            orders_vec.push(order);
            self.decrement_order_count();
        }

        orders_vec
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

    fn make_resting_pco(
        time_priority: SequenceNumber,
        trigger_price: Price,
    ) -> RestingPriceConditionalOrder {
        RestingPriceConditionalOrder::new(
            time_priority,
            0, // LevelId is crate-private; tests are in-crate.
            PriceConditionalOrder::new(
                trigger_price,
                TriggerDirection::AtOrAbove,
                TriggerOrder::Market(MarketOrder::new(Quantity(1), Side::Buy, false)),
            ),
        )
    }

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
    fn drain_orders_drains_valid_orders_and_decrements_order_count() {
        let mut orders: FxHashMap<OrderId, RestingPriceConditionalOrder> = FxHashMap::default();
        let mut level = TriggerPriceLevel::new();

        let o0 = make_resting_pco(SequenceNumber(0), Price(100));
        let o1 = make_resting_pco(SequenceNumber(1), Price(100));
        orders.insert(OrderId(0), o0.clone());
        orders.insert(OrderId(1), o1.clone());

        level.add_order_entry(QueueEntry::new(SequenceNumber(0), OrderId(0)));
        level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(level.order_count(), 2);

        let drained = level.drain_orders(&mut orders);
        assert_eq!(drained, vec![o0, o1]);
        assert!(orders.is_empty());
        assert_eq!(level.order_count(), 0);
        assert!(level.is_empty());
        assert!(level.queue().is_empty());
    }

    #[test]
    fn drain_orders_skips_stale_missing_order_entries_before_valid_orders() {
        let mut orders: FxHashMap<OrderId, RestingPriceConditionalOrder> = FxHashMap::default();
        let mut level = TriggerPriceLevel::new();

        // Stale entry: present in queue, but missing from `orders` map.
        // Important: do NOT increment order_count for this entry (simulates "deferred cleanup").
        level.push(QueueEntry::new(SequenceNumber(0), OrderId(999)));

        // Valid entry: reflected in order_count and in the orders map.
        let o1 = make_resting_pco(SequenceNumber(1), Price(123));
        orders.insert(OrderId(1), o1.clone());
        level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(level.order_count(), 1);

        let drained = level.drain_orders(&mut orders);
        assert_eq!(drained, vec![o1]);
        assert!(orders.is_empty());
        assert_eq!(level.order_count(), 0);
        assert!(level.queue().is_empty());
    }

    #[test]
    #[should_panic]
    fn drain_orders_panics_if_order_count_exceeds_remaining_queue_entries() {
        let mut orders: FxHashMap<OrderId, RestingPriceConditionalOrder> = FxHashMap::default();
        let mut level = TriggerPriceLevel::new();

        // order_count says there's 1 active order...
        level.increment_order_count();
        // ...but the queue has only a stale entry that will be skipped (and will not decrement order_count).
        level.push(QueueEntry::new(SequenceNumber(0), OrderId(0)));

        // This violates internal invariants: the loop will try to pop again from an empty queue.
        let _ = level.drain_orders(&mut orders);
    }
}
