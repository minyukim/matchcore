use crate::{LimitOrder, MarketOrder, PeggedOrder, Side, TimeInForce};

/// Represents a command to submit a new order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitCmd {
    /// The order to submit
    pub order: NewOrder,
}

/// Represents a new order for all order types
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewOrder {
    /// A new market order
    Market(MarketOrder),
    /// A new limit order
    Limit(LimitOrder),
    /// A new pegged order
    Pegged(PeggedOrder),
}

/// Represents the shared core data for all order types
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewOrderCore {
    /// The side of the order
    pub side: Side,
    /// Whether the order is post-only
    pub post_only: bool,
    /// The time in force of the order
    pub time_in_force: TimeInForce,
}
