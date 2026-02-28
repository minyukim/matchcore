use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancelReason {
    /// Insufficient liquidity for market orders and IOC/FOK orders
    InsufficientLiquidity {
        /// The quantity of the order that was requested to be filled
        requested_quantity: u64,
        /// The quantity of the order that was available to be filled
        available_quantity: u64,
    },
    /// The maker side of the order book is empty
    EmptyMakerSide,
    /// The post-only order would remove liquidity
    PostOnlyWouldTake,
}

impl fmt::Display for CancelReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CancelReason::InsufficientLiquidity {
                requested_quantity,
                available_quantity,
            } => write!(
                f,
                "Insufficient liquidity: requested={} available={}",
                requested_quantity, available_quantity
            ),
            CancelReason::EmptyMakerSide => write!(f, "Maker side is empty"),
            CancelReason::PostOnlyWouldTake => {
                write!(f, "Post-only order would remove liquidity")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(
            CancelReason::InsufficientLiquidity {
                requested_quantity: 100,
                available_quantity: 50,
            }
            .to_string(),
            "Insufficient liquidity: requested=100 available=50"
        );
        assert_eq!(
            CancelReason::EmptyMakerSide.to_string(),
            "Maker side is empty"
        );
        assert_eq!(
            CancelReason::PostOnlyWouldTake.to_string(),
            "Post-only order would remove liquidity"
        );
    }
}
