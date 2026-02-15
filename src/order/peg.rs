use crate::order::{Side, TimeInForce};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Reference price type for pegged orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PegReference {
    /// Pegged to best bid price
    BestBid,
    /// Pegged to best ask price
    BestAsk,
    /// Pegged to mid price between bid and ask
    MidPrice,
    /// Pegged to last trade price
    LastTrade,
}

impl fmt::Display for PegReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PegReference::BestBid => write!(f, "BestBid"),
            PegReference::BestAsk => write!(f, "BestAsk"),
            PegReference::MidPrice => write!(f, "MidPrice"),
            PegReference::LastTrade => write!(f, "LastTrade"),
        }
    }
}

/// Pegged order that adjusts based on reference price
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeggedOrder<T = ()>
where
    T: Clone + Copy + Eq + Serialize + core::fmt::Debug,
{
    /// The order ID
    id: u64,
    /// Reference price to track
    reference: PegReference,
    /// The quantity of the order
    qty: u64,
    /// The side of the order (buy or sell)
    side: Side,
    /// Whether the order is post-only
    post_only: bool,
    /// When the order was created
    timestamp: u64,
    /// Time-in-force policy
    time_in_force: TimeInForce,
    /// Additional custom fields
    extra: T,
}

impl<T: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug>
    PeggedOrder<T>
{
    /// Create a new pegged order
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        reference: PegReference,
        qty: u64,
        side: Side,
        post_only: bool,
        timestamp: u64,
        time_in_force: TimeInForce,
        extra: T,
    ) -> Self {
        Self {
            id,
            reference,
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

    /// Get the reference price
    pub fn reference(&self) -> PegReference {
        self.reference
    }

    /// Update the reference price
    pub fn update_reference(&mut self, new_reference: PegReference) {
        self.reference = new_reference;
    }

    /// Get the quantity of the order
    pub fn qty(&self) -> u64 {
        self.qty
    }

    /// Update the quantity of the order
    pub fn update_qty(&mut self, new_qty: u64) {
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
    pub fn match_against(&mut self, incoming_qty: u64) -> (u64, u64) {
        let new_qty = self.qty.saturating_sub(incoming_qty);
        let consumed = self.qty - new_qty;
        let remaining = incoming_qty - consumed;

        self.qty = new_qty;
        (consumed, remaining)
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
    pub fn map_extra<U, F>(&self, f: F) -> PeggedOrder<U>
    where
        F: FnOnce(T) -> U,
        U: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug,
    {
        PeggedOrder::new(
            self.id,
            self.reference,
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
    for PeggedOrder<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pegged: id={} reference={} qty={} side={} post_only={} timestamp={} time_in_force={}",
            self.id,
            self.reference,
            self.qty,
            self.side,
            self.post_only,
            self.timestamp,
            self.time_in_force
        )
    }
}
