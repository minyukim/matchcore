use crate::{Quantity, Side};

/// Market order that is executed immediately and does not reside in the order book
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketOrder {
    /// The quantity of the order
    quantity: Quantity,
    /// The side of the order
    side: Side,
    /// Whether to convert the order to a limit order if it is not filled immediately
    market_to_limit: bool,
}

impl MarketOrder {
    /// Create a new market order
    pub fn new(quantity: Quantity, side: Side, market_to_limit: bool) -> Self {
        Self {
            quantity,
            side,
            market_to_limit,
        }
    }

    /// Get the quantity of the order
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Get the side of the order
    pub fn side(&self) -> Side {
        self.side
    }

    /// Get whether to convert the order to a limit order
    pub fn market_to_limit(&self) -> bool {
        self.market_to_limit
    }
}
