use crate::{Quantity, Side};

use serde::{Deserialize, Serialize};

/// Specification of a market order
/// Note that the market order only has a specification, but not the order itself, it is converted
/// to a limit order if it is not filled immediately and the market_to_limit flag is set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketOrderSpec {
    /// The quantity of the order
    quantity: Quantity,
    /// The side of the order
    side: Side,
    /// Whether to convert the order to a limit order
    /// if it is not filled immediately at the best available price
    market_to_limit: bool,
}

impl MarketOrderSpec {
    /// Create a new market order specification
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
