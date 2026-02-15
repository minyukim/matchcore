use crate::order::{QtyPolicy, Side, TimeInForce};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a generic limit order with additional custom fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order<T = ()>
where
    T: Clone + Copy + Eq + Serialize + core::fmt::Debug,
{
    /// The order ID
    id: u64,
    /// The price of the order
    price: u64,
    /// The quantity policy of the order
    qty: QtyPolicy,
    /// The side of the order (buy or sell)
    side: Side,
    /// Whether the order is post-only
    post_only: bool,
    /// When the order was created, expressed as a Unix timestamp (seconds since epoch).
    timestamp: u64,
    /// Time-in-force policy
    time_in_force: TimeInForce,
    /// Additional custom fields
    extra: T,
}

impl<T: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> Order<T> {
    /// Create a new order
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        price: u64,
        qty: QtyPolicy,
        side: Side,
        post_only: bool,
        timestamp: u64,
        time_in_force: TimeInForce,
        extra: T,
    ) -> Self {
        Self {
            id,
            price,
            qty,
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
        self.qty.visible_qty()
    }

    /// Get the hidden quantity
    pub fn hidden_quantity(&self) -> u64 {
        self.qty.hidden_qty()
    }

    /// Get the replenish size
    pub fn replenish_size(&self) -> u64 {
        self.qty.replenish_size()
    }

    /// Update the quantity of the order
    pub fn update_qty(&mut self, new_qty: QtyPolicy) {
        self.qty = new_qty;
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
    pub fn match_against(&mut self, incoming_qty: u64) -> (u64, u64, u64) {
        match self.qty {
            QtyPolicy::Standard { qty } => {
                let new_qty = qty.saturating_sub(incoming_qty);
                let consumed = qty - new_qty;
                let remaining = incoming_qty - consumed;

                self.qty.update_visible_qty(new_qty);
                (consumed, remaining, 0)
            }
            QtyPolicy::Iceberg { visible_qty, .. } => {
                let new_visible = visible_qty.saturating_sub(incoming_qty);
                let consumed = visible_qty - new_visible;
                let remaining = incoming_qty - consumed;

                self.qty.update_visible_qty(new_visible);
                if new_visible > 0 {
                    (consumed, remaining, 0)
                } else {
                    // Try replenishing the order
                    let replenished = self.qty.replenish();
                    (consumed, remaining, replenished)
                }
            }
        }
    }

    /// Get the extra fields
    pub fn extra(&self) -> &T {
        &self.extra
    }

    /// Get mutable reference to extra fields
    pub fn extra_mut(&mut self) -> &mut T {
        &mut self.extra
    }

    /// Transform the extra fields type using a function
    pub fn map_extra<U, F>(&self, f: F) -> Order<U>
    where
        F: FnOnce(T) -> U,
        U: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
    {
        Order::new(
            self.id,
            self.price,
            self.qty,
            self.side,
            self.post_only,
            self.timestamp,
            self.time_in_force,
            f(self.extra),
        )
    }
}

impl<T: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> fmt::Display
    for Order<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.qty {
            QtyPolicy::Standard { qty } => {
                write!(
                    f,
                    "Standard: id={} price={} qty={} side={} post_only={} timestamp={} time_in_force={}",
                    self.id,
                    self.price,
                    qty,
                    self.side,
                    self.post_only,
                    self.timestamp,
                    self.time_in_force
                )
            }
            QtyPolicy::Iceberg {
                visible_qty,
                hidden_qty,
                replenish_size,
            } => {
                write!(
                    f,
                    "Iceberg: id={} price={} visible_qty={} hidden_qty={} replenish_size={} side={} post_only={} timestamp={} time_in_force={}",
                    self.id,
                    self.price,
                    visible_qty,
                    hidden_qty,
                    replenish_size,
                    self.side,
                    self.post_only,
                    self.timestamp,
                    self.time_in_force
                )
            }
        }
    }
}
