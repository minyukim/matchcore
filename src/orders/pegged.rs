use crate::{PegReference, orders::OrderFlags};

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

/// Pegged order that adjusts based on reference price
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeggedOrder {
    /// The ID of the order
    id: u64,
    /// The specification of the order
    spec: PeggedOrderSpec,
}

impl PeggedOrder {
    /// Create a new pegged order
    pub fn new(id: u64, spec: PeggedOrderSpec) -> Self {
        Self { id, spec }
    }

    /// Get the order ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the specification of the order
    pub fn spec(&self) -> &PeggedOrderSpec {
        &self.spec
    }

    /// Matches this order against an incoming quantity
    ///
    /// Returns the quantity consumed from the incoming order
    pub(crate) fn match_against(&mut self, incoming_quantity: u64) -> u64 {
        let new_quantity = self.quantity.saturating_sub(incoming_quantity);
        let consumed = self.quantity - new_quantity;

        self.quantity = new_quantity;
        consumed
    }
}

impl Deref for PeggedOrder {
    type Target = PeggedOrderSpec;

    fn deref(&self) -> &Self::Target {
        &self.spec
    }
}
impl DerefMut for PeggedOrder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.spec
    }
}

impl fmt::Display for PeggedOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pegged: id={} peg_reference={} quantity={} side={} post_only={} time_in_force={}",
            self.id(),
            self.peg_reference(),
            self.quantity(),
            self.side(),
            self.post_only(),
            self.time_in_force()
        )
    }
}

/// Specification of a pegged order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeggedOrderSpec {
    /// Reference price to track
    peg_reference: PegReference,
    /// The quantity of the order
    quantity: u64,
    /// The flags of the order
    flags: OrderFlags,
}

impl PeggedOrderSpec {
    /// Create a new pegged order specification
    pub fn new(peg_reference: PegReference, quantity: u64, flags: OrderFlags) -> Self {
        Self {
            peg_reference,
            quantity,
            flags,
        }
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
    pub fn quantity(&self) -> u64 {
        self.quantity
    }

    /// Check if the order is filled
    pub fn is_filled(&self) -> bool {
        self.quantity == 0
    }

    /// Update the quantity of the order
    pub(crate) fn update_quantity(&mut self, new_quantity: u64) {
        self.quantity = new_quantity;
    }

    /// Get the flags of the order
    pub fn flags(&self) -> &OrderFlags {
        &self.flags
    }
}

impl Deref for PeggedOrderSpec {
    type Target = OrderFlags;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}
impl DerefMut for PeggedOrderSpec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PegReference, Side, TimeInForce, orders::OrderFlags};

    fn create_pegged_order() -> PeggedOrder {
        PeggedOrder::new(
            0,
            PeggedOrderSpec::new(
                PegReference::Primary,
                20,
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        )
    }

    #[test]
    fn test_id() {
        assert_eq!(create_pegged_order().id(), 0);
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
        assert_eq!(order.quantity(), 20);
        assert!(!order.is_filled());

        order.update_quantity(30);
        assert_eq!(order.quantity(), 30);
        assert!(!order.is_filled());

        order.update_quantity(10);
        assert_eq!(order.quantity(), 10);
        assert!(!order.is_filled());

        order.update_quantity(0);
        assert_eq!(order.quantity(), 0);
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
        assert!(!order.is_expired(1771180000));

        order.update_time_in_force(TimeInForce::Ioc);
        assert!(order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(1771180000));

        order.update_time_in_force(TimeInForce::Fok);
        assert!(order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(1771180000));

        order.update_time_in_force(TimeInForce::Gtd(1771180000 + 1000));
        assert!(!order.is_immediate());
        assert!(order.has_expiry());
        assert!(!order.is_expired(1771180000));
        assert!(order.is_expired(1771180000 + 1000));
    }

    #[test]
    fn test_match_against() {
        let mut order = create_pegged_order();
        assert_eq!(order.quantity(), 20);

        let consumed = order.match_against(2);
        assert_eq!(consumed, 2);
        assert_eq!(order.quantity(), 18);

        let consumed = order.match_against(20);
        assert_eq!(consumed, 18);
        assert_eq!(order.quantity(), 0);

        let consumed = order.match_against(10);
        assert_eq!(consumed, 0);
        assert_eq!(order.quantity(), 0);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let order = create_pegged_order();
        let serialized = serde_json::to_string(&order).unwrap();
        let deserialized: PeggedOrder = serde_json::from_str(&serialized).unwrap();
        assert_eq!(order, deserialized);
    }

    #[test]
    fn test_display() {
        assert_eq!(
            create_pegged_order().to_string(),
            "Pegged: id=0 peg_reference=Primary quantity=20 side=BUY post_only=true time_in_force=GTC"
        );
    }
}
