use crate::{PegReference, QuantityPolicy, Side, TimeInForce};

use serde::{Deserialize, Serialize};

/// Represents a command to submit a new order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitCmd<E = ()> {
    /// The order to submit
    pub order: NewOrder<E>,
}

/// Represents a new order for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewOrder<E = ()> {
    /// A new market order
    Market(NewMarketOrder<E>),
    /// A new limit order
    Limit(NewLimitOrder<E>),
    /// A new pegged order
    Pegged(NewPeggedOrder<E>),
}

/// Represents a new market order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewMarketOrder<E = ()> {
    /// The quantity of the order
    pub quantity: u64,
    /// The side of the order
    pub side: Side,
    /// Whether to convert the order to a limit order
    /// if it is not filled immediately at the best available price
    pub market_to_limit: bool,
    /// Additional custom fields
    pub extra: E,
}

/// Represents a new limit order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewLimitOrder<E = ()> {
    /// The core order data
    pub core: NewOrderCore<E>,
    /// The price of the order
    pub price: u64,
    /// The quantity policy of the order
    pub quantity_policy: QuantityPolicy,
}

/// Represents a new pegged order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPeggedOrder<E = ()> {
    /// The core order data
    pub core: NewOrderCore<E>,
    /// The peg reference type
    pub peg_reference: PegReference,
    /// The quantity of the order
    pub quantity: u64,
}

/// Represents the shared core data for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrderCore<E = ()> {
    /// The side of the order
    pub side: Side,
    /// Whether the order is post-only
    pub post_only: bool,
    /// The time in force of the order
    pub time_in_force: TimeInForce,
    /// Additional custom fields
    pub extra: E,
}
