use crate::order::{Side, TimeInForce};

use serde::{Deserialize, Serialize};

/// Core order data that is common to all order types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// When the order was created, expressed as a Unix timestamp (seconds since epoch).
    timestamp: u64,
    /// Time-in-force policy
    time_in_force: TimeInForce,
    /// Additional custom fields
    extra: E,
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> OrderCore<E> {
    /// Create a new order core
    pub fn new(
        id: u64,
        side: Side,
        post_only: bool,
        timestamp: u64,
        time_in_force: TimeInForce,
        extra: E,
    ) -> Self {
        Self {
            id,
            side,
            post_only,
            timestamp,
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

    /// Get the timestamp
    pub(super) fn timestamp(&self) -> u64 {
        self.timestamp
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
            self.timestamp,
            self.time_in_force,
            f(self.extra),
        )
    }
}
