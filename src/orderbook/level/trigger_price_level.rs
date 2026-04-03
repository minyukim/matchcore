use super::{LevelEntries, QueueEntry};
use crate::{OrderId, Price, PriceConditionalOrder, RestingPriceConditionalOrder};

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
    ) -> Vec<(OrderId, PriceConditionalOrder)> {
        let mut orders_vec = Vec::with_capacity(self.order_count() as usize);

        while !self.is_empty() {
            let queue_entry = self.pop().unwrap();

            let order_id = queue_entry.order_id();
            let Some(order) = orders.get(&order_id) else {
                continue; // stale entry
            };
            if queue_entry.time_priority() != order.time_priority() {
                continue; // stale entry
            }

            orders_vec.push((order_id, orders.remove(&order_id).unwrap().into_inner()));
            self.decrement_order_count();
        }

        orders_vec
    }

    /// Drains all triggered orders at the given price from the level and the provided `orders` map
    pub(crate) fn drain_triggered_orders_at_price(
        &mut self,
        orders: &mut FxHashMap<OrderId, RestingPriceConditionalOrder>,
        price: Price,
    ) -> Vec<(OrderId, PriceConditionalOrder)> {
        let mut triggered_orders = Vec::new();

        while !self.is_empty() {
            let queue_entry = self.pop().unwrap();

            let order_id = queue_entry.order_id();
            let Some(order) = orders.get(&order_id) else {
                continue; // stale entry
            };
            if queue_entry.time_priority() != order.time_priority() {
                continue; // stale entry
            }

            self.decrement_order_count();

            if !order.price_condition().is_met(price) {
                continue;
            }

            let order = orders.remove(&order_id).unwrap();
            triggered_orders.push((order_id, order.into_inner()));
        }

        triggered_orders
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
        direction: TriggerDirection,
    ) -> RestingPriceConditionalOrder {
        RestingPriceConditionalOrder::new(
            time_priority,
            0, // LevelId is crate-private; tests are in-crate.
            PriceConditionalOrder::new(
                PriceCondition::new(trigger_price, direction),
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

        let o0 = make_resting_pco(SequenceNumber(0), Price(100), TriggerDirection::AtOrAbove);
        let o1 = make_resting_pco(SequenceNumber(1), Price(100), TriggerDirection::AtOrAbove);
        orders.insert(OrderId(0), o0.clone());
        orders.insert(OrderId(1), o1.clone());

        level.add_order_entry(QueueEntry::new(SequenceNumber(0), OrderId(0)));
        level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(level.order_count(), 2);

        let drained = level.drain_orders(&mut orders);
        assert_eq!(
            drained,
            vec![(OrderId(0), o0.into_inner()), (OrderId(1), o1.into_inner())]
        );
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
        let o1 = make_resting_pco(SequenceNumber(1), Price(123), TriggerDirection::AtOrAbove);
        orders.insert(OrderId(1), o1.clone());
        level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(level.order_count(), 1);

        let drained = level.drain_orders(&mut orders);
        assert_eq!(drained, vec![(OrderId(1), o1.into_inner())]);
        assert!(orders.is_empty());
        assert_eq!(level.order_count(), 0);
        assert!(level.queue().is_empty());
    }

    #[test]
    fn drain_triggered_orders_at_price_triggers_at_or_above_when_price_meets_threshold() {
        let mut orders: FxHashMap<OrderId, RestingPriceConditionalOrder> = FxHashMap::default();
        let mut level = TriggerPriceLevel::new();

        let o0 = make_resting_pco(SequenceNumber(0), Price(100), TriggerDirection::AtOrAbove);
        let o1 = make_resting_pco(SequenceNumber(1), Price(101), TriggerDirection::AtOrAbove);
        orders.insert(OrderId(0), o0.clone());
        orders.insert(OrderId(1), o1.clone());

        level.add_order_entry(QueueEntry::new(SequenceNumber(0), OrderId(0)));
        level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(level.order_count(), 2);

        let triggered = level.drain_triggered_orders_at_price(&mut orders, Price(101));
        assert_eq!(
            triggered,
            vec![(OrderId(0), o0.into_inner()), (OrderId(1), o1.into_inner())]
        );
        assert!(orders.is_empty());
        assert_eq!(level.order_count(), 0);
        assert!(level.queue().is_empty());
    }

    #[test]
    fn drain_triggered_orders_at_price_triggers_at_or_below_when_price_meets_threshold() {
        let mut orders: FxHashMap<OrderId, RestingPriceConditionalOrder> = FxHashMap::default();
        let mut level = TriggerPriceLevel::new();

        let o0 = make_resting_pco(SequenceNumber(0), Price(100), TriggerDirection::AtOrBelow);
        let o1 = make_resting_pco(SequenceNumber(1), Price(99), TriggerDirection::AtOrBelow);
        orders.insert(OrderId(0), o0.clone());
        orders.insert(OrderId(1), o1.clone());

        level.add_order_entry(QueueEntry::new(SequenceNumber(0), OrderId(0)));
        level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(level.order_count(), 2);

        let triggered = level.drain_triggered_orders_at_price(&mut orders, Price(99));
        assert_eq!(
            triggered,
            vec![(OrderId(0), o0.into_inner()), (OrderId(1), o1.into_inner())]
        );
        assert!(orders.is_empty());
        assert_eq!(level.order_count(), 0);
        assert!(level.queue().is_empty());
    }

    #[test]
    fn drain_triggered_orders_at_price_skips_stale_missing_order_entries_before_valid_orders() {
        let mut orders: FxHashMap<OrderId, RestingPriceConditionalOrder> = FxHashMap::default();
        let mut level = TriggerPriceLevel::new();

        // Stale entry: present in queue, but missing from `orders` map.
        // Important: do NOT increment order_count for this entry.
        level.push(QueueEntry::new(SequenceNumber(0), OrderId(999)));

        let o1 = make_resting_pco(SequenceNumber(1), Price(100), TriggerDirection::AtOrAbove);
        orders.insert(OrderId(1), o1.clone());
        level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));
        assert_eq!(level.order_count(), 1);

        let triggered = level.drain_triggered_orders_at_price(&mut orders, Price(100));
        assert_eq!(triggered, vec![(OrderId(1), o1.into_inner())]);
        assert!(orders.is_empty());
        assert_eq!(level.order_count(), 0);
        assert!(level.queue().is_empty());
    }

    #[test]
    fn drain_triggered_orders_at_price_skips_stale_order_on_time_priority_mismatch() {
        let mut orders: FxHashMap<OrderId, RestingPriceConditionalOrder> = FxHashMap::default();
        let mut level = TriggerPriceLevel::new();

        let o0 = make_resting_pco(SequenceNumber(0), Price(100), TriggerDirection::AtOrAbove);
        let o1 = make_resting_pco(SequenceNumber(1), Price(100), TriggerDirection::AtOrAbove);
        orders.insert(OrderId(0), o0);
        orders.insert(OrderId(1), o1.clone());

        // Stale entry for OrderId(0): mismatched time priority.
        level.add_order_entry(QueueEntry::new(SequenceNumber(999), OrderId(0)));
        // Valid entry for OrderId(1).
        level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)));

        // Mirror realistic flows: the order-count for the stale entry has already been decremented
        // elsewhere (e.g., order update/removal) and the queue cleanup happens later.
        level.mark_order_removed();

        let triggered = level.drain_triggered_orders_at_price(&mut orders, Price(100));
        assert_eq!(triggered, vec![(OrderId(1), o1.into_inner())]);
        assert!(orders.contains_key(&OrderId(0)));
        assert!(!orders.contains_key(&OrderId(1)));
        assert!(level.queue().is_empty());
    }

    #[test]
    fn drain_triggered_orders_at_price_dequeues_non_triggered_orders_but_leaves_them_in_orders_map()
    {
        let mut orders: FxHashMap<OrderId, RestingPriceConditionalOrder> = FxHashMap::default();
        let mut level = TriggerPriceLevel::new();

        let o0 = make_resting_pco(SequenceNumber(0), Price(101), TriggerDirection::AtOrAbove);
        orders.insert(OrderId(0), o0.clone());
        level.add_order_entry(QueueEntry::new(SequenceNumber(0), OrderId(0)));
        assert_eq!(level.order_count(), 1);

        // Price has not reached trigger for AtOrAbove.
        let triggered = level.drain_triggered_orders_at_price(&mut orders, Price(100));
        assert!(triggered.is_empty());

        // Current behavior: the queue entry is popped and order_count is decremented even if not triggered,
        // but the order remains in the map (not removed).
        assert!(orders.contains_key(&OrderId(0)));
        assert_eq!(level.order_count(), 0);
        assert!(level.queue().is_empty());
    }
}
