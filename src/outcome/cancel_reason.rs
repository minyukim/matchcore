use crate::Quantity;

use std::fmt;

use serde::{Deserialize, Serialize};

/// Reason for cancelling an order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancelReason {
    /// Insufficient liquidity for immediate orders
    InsufficientLiquidity {
        /// The quantity of the order that was available to be filled
        available: Quantity,
    },
    /// The post-only order would remove liquidity
    PostOnlyWouldTake,
}

impl fmt::Display for CancelReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CancelReason::InsufficientLiquidity { available } => {
                write!(f, "insufficient liquidity: available={}", available)
            }
            CancelReason::PostOnlyWouldTake => {
                write!(f, "post-only order would remove liquidity")
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
                available: Quantity(50),
            }
            .to_string(),
            "insufficient liquidity: available=50"
        );
        assert_eq!(
            CancelReason::PostOnlyWouldTake.to_string(),
            "post-only order would remove liquidity"
        );
    }
}
