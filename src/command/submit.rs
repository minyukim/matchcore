use crate::{LimitOrder, MarketOrder, PeggedOrderSpec, Side, TimeInForce};

use serde::{Deserialize, Serialize};

/// Represents a command to submit a new order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitCmd {
    /// The order to submit
    pub order: NewOrder,
}

/// Represents a new order for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewOrder {
    /// A new market order
    Market(MarketOrder),
    /// A new limit order
    Limit(LimitOrder),
    /// A new pegged order
    Pegged(PeggedOrderSpec),
}

/// Represents the shared core data for all order types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewOrderCore {
    /// The side of the order
    pub side: Side,
    /// Whether the order is post-only
    pub post_only: bool,
    /// The time in force of the order
    pub time_in_force: TimeInForce,
}
