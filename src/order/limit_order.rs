use crate::{QuantityPolicy, Side, TimeInForce, order::OrderCore};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Generic limit order with various configuration options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimitOrder<E = ()>
where
    E: Clone + Copy + Eq + Serialize + core::fmt::Debug,
{
    /// The core order data
    core: OrderCore<E>,
    /// The price of the order
    price: u64,
    /// The quantity policy of the order
    quantity_policy: QuantityPolicy,
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug>
    LimitOrder<E>
{
    /// Create a new order
    pub fn new(core: OrderCore<E>, price: u64, quantity_policy: QuantityPolicy) -> Self {
        Self {
            core,
            price,
            quantity_policy,
        }
    }

    /// Get the order ID
    pub fn id(&self) -> u64 {
        self.core.id()
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
        self.quantity_policy.visible_quantity()
    }

    /// Get the hidden quantity
    pub fn hidden_quantity(&self) -> u64 {
        self.quantity_policy.hidden_quantity()
    }

    /// Get the replenish quantity
    pub fn replenish_quantity(&self) -> u64 {
        self.quantity_policy.replenish_quantity()
    }

    /// Get the total quantity of the order
    pub fn total_quantity(&self) -> u64 {
        self.quantity_policy.total_quantity()
    }

    /// Check if the order is filled
    pub fn is_filled(&self) -> bool {
        self.quantity_policy.is_filled()
    }

    /// Update the quantity policy of the order
    pub fn update_quantity_policy(&mut self, new_quantity_policy: QuantityPolicy) {
        self.quantity_policy = new_quantity_policy;
    }

    /// Get the order side
    pub fn side(&self) -> Side {
        self.core.side()
    }

    /// Check if this is a post-only order
    pub fn is_post_only(&self) -> bool {
        self.core.is_post_only()
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> u64 {
        self.core.timestamp()
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
    pub fn update_time_in_force(&mut self, new_time_in_force: TimeInForce) {
        self.core.update_time_in_force(new_time_in_force);
    }

    /// Matches this order against an incoming quantity
    ///
    /// Returns a tuple containing:
    /// - The quantity consumed from the incoming order
    /// - The quantity that was replenished (for iceberg orders)
    pub fn match_against(&mut self, incoming_quantity: u64) -> (u64, u64) {
        match self.quantity_policy {
            QuantityPolicy::Standard { quantity } => {
                let new_quantity = quantity.saturating_sub(incoming_quantity);
                let consumed = quantity - new_quantity;

                self.quantity_policy.update_visible_quantity(new_quantity);
                (consumed, 0)
            }
            QuantityPolicy::Iceberg {
                visible_quantity, ..
            } => {
                let new_visible = visible_quantity.saturating_sub(incoming_quantity);
                let consumed = visible_quantity - new_visible;

                self.quantity_policy.update_visible_quantity(new_visible);
                if new_visible > 0 {
                    (consumed, 0)
                } else {
                    // Try replenishing the order
                    let replenished = self.quantity_policy.replenish();
                    (consumed, replenished)
                }
            }
        }
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
    pub fn map_extra<U, F>(&self, f: F) -> LimitOrder<U>
    where
        F: FnOnce(E) -> U,
        U: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
    {
        LimitOrder::new(self.core.map_extra(f), self.price, self.quantity_policy)
    }
}

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> fmt::Display
    for LimitOrder<E>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.quantity_policy {
            QuantityPolicy::Standard { quantity } => {
                write!(
                    f,
                    "Standard: id={} price={} quantity={} side={} post_only={} timestamp={} time_in_force={}",
                    self.core.id(),
                    self.price,
                    quantity,
                    self.core.side(),
                    self.core.is_post_only(),
                    self.core.timestamp(),
                    self.core.time_in_force()
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
                    self.core.id(),
                    self.price,
                    visible_quantity,
                    hidden_quantity,
                    replenish_quantity,
                    self.core.side(),
                    self.core.is_post_only(),
                    self.core.timestamp(),
                    self.core.time_in_force()
                )
            }
        }
    }
}
