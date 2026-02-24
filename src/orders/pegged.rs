use crate::{PegReference, Side, TimeInForce, orders::OrderCore};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Pegged order that adjusts based on reference price
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeggedOrder<E = ()>
where
    E: Clone + Copy + Eq + Serialize + core::fmt::Debug,
{
    /// The core order data
    core: OrderCore<E>,
    /// Reference price to track
    peg_reference: PegReference,
    /// The quantity of the order
    quantity: u64,
}

#[allow(unused)]
impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug>
    PeggedOrder<E>
{
    /// Create a new pegged order
    pub fn new(core: OrderCore<E>, peg_reference: PegReference, quantity: u64) -> Self {
        Self {
            core,
            peg_reference,
            quantity,
        }
    }

    /// Get the order ID
    pub fn id(&self) -> u64 {
        self.core.id()
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

    /// Get the order side
    pub fn side(&self) -> Side {
        self.core.side()
    }

    /// Check if this is a post-only order
    pub fn is_post_only(&self) -> bool {
        self.core.is_post_only()
    }

    /// Update the post-only flag
    pub(crate) fn update_post_only(&mut self, new_post_only: bool) {
        self.core.update_post_only(new_post_only);
    }

    /// Get the time in force
    pub fn time_in_force(&self) -> TimeInForce {
        self.core.time_in_force()
    }

    /// Check if the order should be canceled after attempting to match
    pub fn is_immediate(&self) -> bool {
        self.core.is_immediate()
    }

    /// Check if the order has an expiry time
    pub fn has_expiry(&self) -> bool {
        self.core.has_expiry()
    }

    /// Check if the order is expired at a given timestamp
    pub fn is_expired(&self, timestamp: u64) -> bool {
        self.core.is_expired(timestamp)
    }

    /// Update the time in force
    pub(crate) fn update_time_in_force(&mut self, new_time_in_force: TimeInForce) {
        self.core.update_time_in_force(new_time_in_force);
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

    /// Get the extra fields
    pub fn extra(&self) -> &E {
        self.core.extra()
    }

    /// Get mutable reference to extra fields
    pub fn extra_mut(&mut self) -> &mut E {
        self.core.extra_mut()
    }

    /// Transform the extra fields type using a function
    pub fn map_extra<G, F>(&self, f: F) -> PeggedOrder<G>
    where
        F: FnOnce(E) -> G,
        G: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
    {
        PeggedOrder::new(self.core.map_extra(f), self.peg_reference, self.quantity)
    }
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> fmt::Display
    for PeggedOrder<E>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pegged: id={} peg_reference={} quantity={} side={} post_only={} time_in_force={}",
            self.core.id(),
            self.peg_reference,
            self.quantity,
            self.core.side(),
            self.core.is_post_only(),
            self.core.time_in_force()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PegReference, Side, TimeInForce, orders::OrderCore};

    fn create_pegged_order() -> PeggedOrder {
        PeggedOrder::new(
            OrderCore::new(0, Side::Buy, true, TimeInForce::Gtc, ()),
            PegReference::Primary,
            20,
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
    fn test_is_post_only() {
        assert!(create_pegged_order().is_post_only());
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
