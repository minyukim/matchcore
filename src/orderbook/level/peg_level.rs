use super::{LevelEntries, QueueEntry};
use crate::{OrderId, PegReference, Quantity, RestingPeggedOrder, SequenceNumber};

use std::ops::{Deref, DerefMut};

use rustc_hash::FxHashMap;

/// Maker side array for the primary peg reference
pub(crate) static MAKER_ARRAY_PRIMARY: [PegReference; 1] = [PegReference::Primary];
/// Maker side array for the primary and mid price peg references
pub(crate) static MAKER_ARRAY_PRIMARY_MID_PRICE: [PegReference; 2] =
    [PegReference::Primary, PegReference::MidPrice];

/// Peg level that manages the status of the orders with the same peg reference.
/// It does not store the orders themselves, but only the time priority information of the orders.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct PegLevel {
    /// The sequence number at which the peg level was last repriced
    pub(crate) repriced_at: SequenceNumber,
    /// Total quantity at this peg level
    pub(crate) quantity: Quantity,
    /// The level entries for this peg level
    level_entries: LevelEntries,
}

impl Default for PegLevel {
    fn default() -> Self {
        Self::new()
    }
}

impl PegLevel {
    /// Create a new peg level
    pub fn new() -> Self {
        Self {
            repriced_at: SequenceNumber(0),
            quantity: Quantity(0),
            level_entries: LevelEntries::new(),
        }
    }

    /// Get the sequence number at which the peg level was last repriced
    pub fn repriced_at(&self) -> SequenceNumber {
        self.repriced_at
    }

    /// Get the quantity at this peg level
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Get the level entries for this peg level
    pub fn level_entries(&self) -> &LevelEntries {
        &self.level_entries
    }

    /// Add an order entry to the peg level
    pub(crate) fn add_order_entry(&mut self, queue_entry: QueueEntry, quantity: Quantity) {
        self.quantity += quantity;

        self.push(queue_entry);
        self.increment_order_count();
    }

    /// Mark an order as removed from the peg level
    /// Note that it does not remove the queue entry from the queue.
    /// The stale queue entry will be cleaned up when the order is peeked from the queue.
    pub(crate) fn mark_order_removed(&mut self, quantity: Quantity) {
        self.quantity -= quantity;
        self.decrement_order_count();
    }

    /// Pop the first queue entry from the peg level and remove the order from the order book
    /// If the peg level is empty, do nothing
    /// Note that it does not update the quantity of the peg level
    pub(crate) fn remove_head_order(
        &mut self,
        orders: &mut FxHashMap<OrderId, RestingPeggedOrder>,
    ) {
        let Some(queue_entry) = self.pop() else {
            return;
        };
        orders.remove(&queue_entry.order_id());
        self.decrement_order_count();
    }
}

impl Deref for PegLevel {
    type Target = LevelEntries;

    fn deref(&self) -> &Self::Target {
        &self.level_entries
    }
}
impl DerefMut for PegLevel {
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
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.order_count(), 0);

        peg_level.increment_order_count();
        assert_eq!(peg_level.order_count(), 1);

        peg_level.decrement_order_count();
        assert_eq!(peg_level.order_count(), 0);
    }

    #[test]
    fn test_add_order_entry_and_mark_order_removed() {
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.quantity(), Quantity(0));
        assert_eq!(peg_level.order_count(), 0);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(0), OrderId(0)), Quantity(10));
        assert_eq!(peg_level.quantity(), Quantity(10));
        assert_eq!(peg_level.order_count(), 1);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(1), OrderId(1)), Quantity(20));
        assert_eq!(peg_level.quantity(), Quantity(30));
        assert_eq!(peg_level.order_count(), 2);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(2), OrderId(2)), Quantity(30));
        assert_eq!(peg_level.quantity(), Quantity(60));
        assert_eq!(peg_level.order_count(), 3);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(3), OrderId(3)), Quantity(40));
        assert_eq!(peg_level.quantity(), Quantity(100));
        assert_eq!(peg_level.order_count(), 4);

        peg_level.add_order_entry(QueueEntry::new(SequenceNumber(4), OrderId(4)), Quantity(50));
        assert_eq!(peg_level.quantity(), Quantity(150));
        assert_eq!(peg_level.order_count(), 5);

        peg_level.mark_order_removed(Quantity(10));
        assert_eq!(peg_level.quantity(), Quantity(140));
        assert_eq!(peg_level.order_count(), 4);

        peg_level.mark_order_removed(Quantity(20));
        assert_eq!(peg_level.quantity(), Quantity(120));
        assert_eq!(peg_level.order_count(), 3);

        peg_level.mark_order_removed(Quantity(30));
        assert_eq!(peg_level.quantity(), Quantity(90));
        assert_eq!(peg_level.order_count(), 2);

        peg_level.mark_order_removed(Quantity(40));
        assert_eq!(peg_level.quantity(), Quantity(50));
        assert_eq!(peg_level.order_count(), 1);

        peg_level.mark_order_removed(Quantity(50));
        assert_eq!(peg_level.quantity(), Quantity(0));
        assert_eq!(peg_level.order_count(), 0);
    }

    #[test]
    fn test_remove_head_order() {
        let mut pegged_orders = FxHashMap::default();

        let mut peg_level = PegLevel::new();
        assert!(peg_level.peek().is_none());

        pegged_orders.insert(
            OrderId(0),
            RestingPeggedOrder::new(
                SequenceNumber(0),
                PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        peg_level.add_order_entry(
            QueueEntry::new(SequenceNumber(0), OrderId(0)),
            Quantity(100),
        );
        assert_eq!(
            peg_level.peek(),
            Some(QueueEntry::new(SequenceNumber(0), OrderId(0)))
        );

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek().is_none());

        pegged_orders.insert(
            OrderId(1),
            RestingPeggedOrder::new(
                SequenceNumber(1),
                PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        peg_level.add_order_entry(
            QueueEntry::new(SequenceNumber(1), OrderId(1)),
            Quantity(100),
        );
        assert_eq!(
            peg_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );

        pegged_orders.insert(
            OrderId(2),
            RestingPeggedOrder::new(
                SequenceNumber(2),
                PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
                ),
            ),
        );
        peg_level.add_order_entry(
            QueueEntry::new(SequenceNumber(2), OrderId(2)),
            Quantity(100),
        );
        assert_eq!(
            peg_level.peek(),
            Some(QueueEntry::new(SequenceNumber(1), OrderId(1)))
        );

        peg_level.remove_head_order(&mut pegged_orders);
        assert_eq!(
            peg_level.peek(),
            Some(QueueEntry::new(SequenceNumber(2), OrderId(2)))
        );

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek().is_none());
    }
}
