use crate::{LimitOrder, MarketOrder, PeggedOrder};

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
