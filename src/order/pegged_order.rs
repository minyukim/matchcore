use crate::order::{OrderCore, Side, TimeInForce};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Pegged order that adjusts based on reference price
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug>
    PeggedOrder<E>
{
    /// Create a new pegged order
    #[allow(clippy::too_many_arguments)]
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
    pub fn update_peg_reference(&mut self, new_peg_reference: PegReference) {
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
    pub fn update_quantity(&mut self, new_quantity: u64) {
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
    /// - The remaining quantity of the incoming order
    pub fn match_against(&mut self, incoming_quantity: u64) -> (u64, u64) {
        let new_quantity = self.quantity.saturating_sub(incoming_quantity);
        let consumed = self.quantity - new_quantity;
        let remaining = incoming_quantity - consumed;

        self.quantity = new_quantity;
        (consumed, remaining)
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
            "Pegged: id={} peg_reference={} quantity={} side={} post_only={} timestamp={} time_in_force={}",
            self.core.id(),
            self.peg_reference,
            self.quantity,
            self.core.side(),
            self.core.is_post_only(),
            self.core.timestamp(),
            self.core.time_in_force()
        )
    }
}

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

impl PegReference {
    pub const COUNT: usize = 4;

    #[inline]
    pub const fn as_index(&self) -> usize {
        match self {
            PegReference::BestBid => 0,
            PegReference::BestAsk => 1,
            PegReference::MidPrice => 2,
            PegReference::LastTrade => 3,
        }
    }
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
