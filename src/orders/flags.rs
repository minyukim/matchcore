use crate::{Side, TimeInForce};

use serde::{Deserialize, Serialize};

/// Flags that are common to all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderFlags {
    /// The side of the order (buy or sell)
    side: Side,
    /// Whether the order is post-only
    post_only: bool,
    /// Time-in-force policy
    time_in_force: TimeInForce,
}

impl OrderFlags {
    /// Create a new order flags
    pub fn new(side: Side, post_only: bool, time_in_force: TimeInForce) -> Self {
        Self {
            side,
            post_only,
            time_in_force,
        }
    }

    /// Get the order side
    pub fn side(&self) -> Side {
        self.side
    }

    /// Check if this is a post-only order
    pub fn is_post_only(&self) -> bool {
        self.post_only
    }

    /// Update the post-only flag
    pub(crate) fn update_post_only(&mut self, new_post_only: bool) {
        self.post_only = new_post_only;
    }

    /// Get the time in force
    pub fn time_in_force(&self) -> TimeInForce {
        self.time_in_force
    }

    /// Check if the order should be canceled after attempting to match
    pub fn is_immediate(&self) -> bool {
        self.time_in_force.is_immediate()
    }

    /// Check if the order has an expiry time
    pub fn has_expiry(&self) -> bool {
        self.time_in_force.has_expiry()
    }

    /// Check if the order is expired at a given timestamp
    pub fn is_expired(&self, timestamp: u64) -> bool {
        self.time_in_force.is_expired(timestamp)
    }

    /// Update the time in force
    pub(crate) fn update_time_in_force(&mut self, new_time_in_force: TimeInForce) {
        self.time_in_force = new_time_in_force;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Side, TimeInForce};

    fn create_order_flags() -> OrderFlags {
        OrderFlags::new(Side::Buy, true, TimeInForce::Gtc)
    }

    #[test]
    fn test_side() {
        assert_eq!(create_order_flags().side(), Side::Buy);
    }

    #[test]
    fn test_is_post_only() {
        assert!(create_order_flags().is_post_only());
    }

    #[test]
    fn test_time_in_force() {
        let mut order = create_order_flags();
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
        let order = create_order_flags();
        let serialized = serde_json::to_string(&order).unwrap();
        let deserialized: OrderFlags = serde_json::from_str(&serialized).unwrap();
        assert_eq!(order, deserialized);
    }
}
