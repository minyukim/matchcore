use crate::{Side, TimeInForce};

use serde::{Deserialize, Serialize};

/// Core order data that is common to all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderCore<E = ()>
where
    E: Clone + Copy + Eq + Serialize + core::fmt::Debug,
{
    /// The order ID
    id: u64,
    /// The side of the order (buy or sell)
    side: Side,
    /// Whether the order is post-only
    post_only: bool,
    /// Time-in-force policy
    time_in_force: TimeInForce,
    /// Additional custom fields
    extra: E,
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> OrderCore<E> {
    /// Create a new order core
    pub fn new(id: u64, side: Side, post_only: bool, time_in_force: TimeInForce, extra: E) -> Self {
        Self {
            id,
            side,
            post_only,
            time_in_force,
            extra,
        }
    }

    /// Get the order ID
    pub(super) fn id(&self) -> u64 {
        self.id
    }

    /// Get the order side
    pub(super) fn side(&self) -> Side {
        self.side
    }

    /// Check if this is a post-only order
    pub(super) fn is_post_only(&self) -> bool {
        self.post_only
    }

    /// Update the post-only flag
    pub(super) fn update_post_only(&mut self, new_post_only: bool) {
        self.post_only = new_post_only;
    }

    /// Get the time in force
    pub(super) fn time_in_force(&self) -> TimeInForce {
        self.time_in_force
    }

    /// Check if the order should be canceled after attempting to match
    pub(super) fn is_immediate(&self) -> bool {
        self.time_in_force.is_immediate()
    }

    /// Check if the order has an expiry time
    pub(super) fn has_expiry(&self) -> bool {
        self.time_in_force.has_expiry()
    }

    /// Check if the order is expired at a given timestamp
    pub(super) fn is_expired(&self, timestamp: u64) -> bool {
        self.time_in_force.is_expired(timestamp)
    }

    /// Update the time in force
    pub(super) fn update_time_in_force(&mut self, new_time_in_force: TimeInForce) {
        self.time_in_force = new_time_in_force;
    }

    /// Get the extra fields
    pub(super) fn extra(&self) -> &E {
        &self.extra
    }

    /// Get mutable reference to extra fields
    pub(super) fn extra_mut(&mut self) -> &mut E {
        &mut self.extra
    }

    /// Transform the extra fields type using a function
    pub(super) fn map_extra<U, F>(&self, f: F) -> OrderCore<U>
    where
        F: FnOnce(E) -> U,
        U: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
    {
        OrderCore::new(
            self.id,
            self.side,
            self.post_only,
            self.time_in_force,
            f(self.extra),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Side, TimeInForce};

    fn create_order_core() -> OrderCore {
        OrderCore::new(0, Side::Buy, true, TimeInForce::Gtc, ())
    }

    #[test]
    fn test_id() {
        assert_eq!(create_order_core().id(), 0);
    }

    #[test]
    fn test_side() {
        assert_eq!(create_order_core().side(), Side::Buy);
    }

    #[test]
    fn test_is_post_only() {
        assert!(create_order_core().is_post_only());
    }

    #[test]
    fn test_time_in_force() {
        let mut order = create_order_core();
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
    fn test_roundtrip_serialization() {
        let order = create_order_core();
        let serialized = serde_json::to_string(&order).unwrap();
        let deserialized: OrderCore = serde_json::from_str(&serialized).unwrap();
        assert_eq!(order, deserialized);
    }
}
