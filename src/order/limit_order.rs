use crate::order::{QuantityPolicy, Side, TimeInForce};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Generic limit order with various configuration options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order<E = ()>
where
    E: Clone + Copy + Eq + Serialize + core::fmt::Debug,
{
    /// The order ID
    id: u64,
    /// The price of the order
    price: u64,
    /// The quantity policy of the order
    quantity: QuantityPolicy,
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

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> Order<E> {
    /// Create a new order
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        price: u64,
        quantity: QuantityPolicy,
        side: Side,
        post_only: bool,
        timestamp: u64,
        time_in_force: TimeInForce,
        extra: E,
    ) -> Self {
        Self {
            id,
            price,
            quantity,
            side,
            post_only,
            timestamp,
            time_in_force,
            extra,
        }
    }

    /// Get the order ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the price
    pub fn price(&self) -> u64 {
        self.price
    }

    /// Update the price of the order
    pub fn update_price(&mut self, new_price: u64) {
        self.price = new_price;
    }

    /// Get the visible quantity
    pub fn visible_quantity(&self) -> u64 {
        self.quantity.visible_quantity()
    }

    /// Get the hidden quantity
    pub fn hidden_quantity(&self) -> u64 {
        self.quantity.hidden_quantity()
    }

    /// Get the replenish quantity
    pub fn replenish_quantity(&self) -> u64 {
        self.quantity.replenish_quantity()
    }

    /// Update the quantity of the order
    pub fn update_quantity(&mut self, new_quantity: QuantityPolicy) {
        self.quantity = new_quantity;
    }

    /// Get the order side
    pub fn side(&self) -> Side {
        self.side
    }

    /// Check if this is a post-only order
    pub fn is_post_only(&self) -> bool {
        self.post_only
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> u64 {
        self.timestamp
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
    pub fn update_time_in_force(&mut self, new_time_in_force: TimeInForce) {
        self.time_in_force = new_time_in_force;
    }

    /// Matches this order against an incoming quantity
    ///
    /// Returns a tuple containing:
    /// - The quantity consumed from the incoming order
    /// - The remaining quantity of the incoming order
    /// - The quantity that was replenished (for iceberg orders)
    pub fn match_against(&mut self, incoming_quantity: u64) -> (u64, u64, u64) {
        match self.quantity {
            QuantityPolicy::Standard { quantity } => {
                let new_quantity = quantity.saturating_sub(incoming_quantity);
                let consumed = quantity - new_quantity;
                let remaining = incoming_quantity - consumed;

                self.quantity.update_visible_quantity(new_quantity);
                (consumed, remaining, 0)
            }
            QuantityPolicy::Iceberg {
                visible_quantity, ..
            } => {
                let new_visible = visible_quantity.saturating_sub(incoming_quantity);
                let consumed = visible_quantity - new_visible;
                let remaining = incoming_quantity - consumed;

                self.quantity.update_visible_quantity(new_visible);
                if new_visible > 0 {
                    (consumed, remaining, 0)
                } else {
                    // Try replenishing the order
                    let replenished = self.quantity.replenish();
                    (consumed, remaining, replenished)
                }
            }
        }
    }

    /// Get the extra fields
    pub fn extra(&self) -> &E {
        &self.extra
    }

    /// Get mutable reference to extra fields
    pub fn extra_mut(&mut self) -> &mut E {
        &mut self.extra
    }

    /// Transform the extra fields type using a function
    pub fn map_extra<U, F>(&self, f: F) -> Order<U>
    where
        F: FnOnce(E) -> U,
        U: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
    {
        Order::new(
            self.id,
            self.price,
            self.quantity,
            self.side,
            self.post_only,
            self.timestamp,
            self.time_in_force,
            f(self.extra),
        )
    }
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> fmt::Display
    for Order<E>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.quantity {
            QuantityPolicy::Standard { quantity } => {
                write!(
                    f,
                    "Standard: id={} price={} quantity={} side={} post_only={} timestamp={} time_in_force={}",
                    self.id,
                    self.price,
                    quantity,
                    self.side,
                    self.post_only,
                    self.timestamp,
                    self.time_in_force
                )
            }
            QuantityPolicy::Iceberg {
                visible_quantity,
                hidden_quantity,
                replenish_quantity,
            } => {
                write!(
                    f,
                    "Iceberg: id={} price={} visible_quantity={} hidden_quantity={} replenish_quantity={} side={} post_only={} timestamp={} time_in_force={}",
                    self.id,
                    self.price,
                    visible_quantity,
                    hidden_quantity,
                    replenish_quantity,
                    self.side,
                    self.post_only,
                    self.timestamp,
                    self.time_in_force
                )
            }
        }
    }
}
