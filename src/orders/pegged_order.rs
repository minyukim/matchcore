use super::{OrderFlags, OrderKind};
use crate::{PegReference, Quantity, SequenceNumber};

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

/// Represents a pegged order resting in the order book
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestingPeggedOrder {
    /// The time priority of the order
    time_priority: SequenceNumber,
    /// The pegged order
    order: PeggedOrder,
}

impl RestingPeggedOrder {
    /// Create a new resting pegged order
    pub fn new(time_priority: SequenceNumber, order: PeggedOrder) -> Self {
        Self {
            time_priority,
            order,
        }
    }

    /// Get the time priority of the order
    pub fn time_priority(&self) -> SequenceNumber {
        self.time_priority
    }

    /// Update the time priority of the order
    pub(crate) fn update_time_priority(&mut self, new_time_priority: SequenceNumber) {
        self.time_priority = new_time_priority;
    }

    /// Get the pegged order
    pub fn order(&self) -> &PeggedOrder {
        &self.order
    }

    /// Convert the resting pegged order into a pegged order
    pub fn into_order(self) -> PeggedOrder {
        self.order
    }
}

impl Deref for RestingPeggedOrder {
    type Target = PeggedOrder;

    fn deref(&self) -> &Self::Target {
        &self.order
    }
}
impl DerefMut for RestingPeggedOrder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.order
    }
}

/// Pegged order that adjusts based on reference price
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeggedOrder {
    /// Reference price to track
    peg_reference: PegReference,
    /// The quantity of the order
    quantity: Quantity,
    /// The flags of the order
    flags: OrderFlags,
}

impl PeggedOrder {
    /// Create a new pegged order
    pub fn new(peg_reference: PegReference, quantity: Quantity, flags: OrderFlags) -> Self {
        Self {
            peg_reference,
            quantity,
            flags,
        }
    }

    /// Get the kind of the order
    pub fn kind(&self) -> OrderKind {
        OrderKind::Pegged
    }

    /// Get the peg reference type
    pub fn peg_reference(&self) -> PegReference {
        self.peg_reference
    }

    /// Update the peg reference type
    pub(crate) fn update_peg_reference(&mut self, new_peg_reference: PegReference) {
        self.peg_reference = new_peg_reference;
    }

    /// Get the quantity of the order
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Check if the order is filled
    pub fn is_filled(&self) -> bool {
        self.quantity.is_zero()
    }

    /// Update the quantity of the order
    pub(crate) fn update_quantity(&mut self, new_quantity: Quantity) {
        self.quantity = new_quantity;
    }

    /// Get the flags of the order
    pub fn flags(&self) -> &OrderFlags {
        &self.flags
    }

    /// Match this order against an incoming quantity
    ///
    /// Returns the quantity consumed from the incoming order
    pub(crate) fn match_against(&mut self, incoming_quantity: Quantity) -> Quantity {
        let new_quantity = self.quantity.saturating_sub(incoming_quantity);
        let consumed = self.quantity - new_quantity;

        self.quantity = new_quantity;
        consumed
    }
}

impl Deref for PeggedOrder {
    type Target = OrderFlags;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}
impl DerefMut for PeggedOrder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flags
    }
}

impl fmt::Display for PeggedOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pegged: peg_reference={} quantity={} side={} post_only={} time_in_force={}",
            self.peg_reference(),
            self.quantity(),
            self.side(),
            self.post_only(),
            self.time_in_force()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PegReference, Quantity, Side, TimeInForce, Timestamp, orders::OrderFlags};

    fn create_pegged_order() -> PeggedOrder {
        PeggedOrder::new(
            PegReference::Primary,
            Quantity(20),
            OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
        )
    }

    #[test]
    fn test_time_priority() {
        let mut order = RestingPeggedOrder::new(SequenceNumber(0), create_pegged_order());
        assert_eq!(order.time_priority(), SequenceNumber(0));

        order.update_time_priority(SequenceNumber(1));
        assert_eq!(order.time_priority(), SequenceNumber(1));
    }

    #[test]
    fn test_peg_reference() {
        let mut order = create_pegged_order();
        assert_eq!(order.peg_reference(), PegReference::Primary);

        order.update_peg_reference(PegReference::Market);
        assert_eq!(order.peg_reference(), PegReference::Market);

        order.update_peg_reference(PegReference::MidPrice);
        assert_eq!(order.peg_reference(), PegReference::MidPrice);

        order.update_peg_reference(PegReference::Primary);
        assert_eq!(order.peg_reference(), PegReference::Primary);
    }

    #[test]
    fn test_quantity() {
        let mut order = create_pegged_order();
        assert_eq!(order.quantity(), Quantity(20));
        assert!(!order.is_filled());

        order.update_quantity(Quantity(30));
        assert_eq!(order.quantity(), Quantity(30));
        assert!(!order.is_filled());

        order.update_quantity(Quantity(10));
        assert_eq!(order.quantity(), Quantity(10));
        assert!(!order.is_filled());

        order.update_quantity(Quantity(0));
        assert_eq!(order.quantity(), Quantity(0));
        assert!(order.is_filled());
    }

    #[test]
    fn test_side() {
        assert_eq!(create_pegged_order().side(), Side::Buy);
    }

    #[test]
    fn test_post_only() {
        assert!(create_pegged_order().post_only());
    }

    #[test]
    fn test_time_in_force() {
        let mut order = create_pegged_order();
        assert_eq!(order.time_in_force(), TimeInForce::Gtc);
        assert!(!order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(Timestamp(1771180000)));

        order.update_time_in_force(TimeInForce::Ioc);
        assert!(order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(Timestamp(1771180000)));

        order.update_time_in_force(TimeInForce::Fok);
        assert!(order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(Timestamp(1771180000)));

        order.update_time_in_force(TimeInForce::Gtd(Timestamp(1771180000 + 1000)));
        assert!(!order.is_immediate());
        assert!(order.has_expiry());
        assert!(!order.is_expired(Timestamp(1771180000)));
        assert!(order.is_expired(Timestamp(1771180000 + 1000)));
    }

    #[test]
    fn test_match_against() {
        let mut order = create_pegged_order();
        assert_eq!(order.quantity(), Quantity(20));

        let consumed = order.match_against(Quantity(2));
        assert_eq!(consumed, Quantity(2));
        assert_eq!(order.quantity(), Quantity(18));

        let consumed = order.match_against(Quantity(20));
        assert_eq!(consumed, Quantity(18));
        assert_eq!(order.quantity(), Quantity(0));

        let consumed = order.match_against(Quantity(10));
        assert_eq!(consumed, Quantity(0));
        assert_eq!(order.quantity(), Quantity(0));
    }

    #[test]
    fn test_display() {
        assert_eq!(
            create_pegged_order().to_string(),
            "Pegged: peg_reference=Primary quantity=20 side=BUY post_only=true time_in_force=GTC"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_roundtrip_serialization() {
        let order = create_pegged_order();
        let serialized = serde_json::to_string(&order).unwrap();
        let deserialized: PeggedOrder = serde_json::from_str(&serialized).unwrap();
        assert_eq!(order, deserialized);
    }
}
