use crate::Quantity;

use std::fmt;

/// Reason for the order cancellation
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelReason {
    /// Insufficient liquidity for immediate orders
    InsufficientLiquidity {
        /// The quantity of the order that was requested
        requested: Quantity,
        /// The quantity of the order that was available to be filled
        available: Quantity,
    },
    /// The post-only order would remove liquidity
    PostOnlyWouldTake,
}

impl fmt::Display for CancelReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CancelReason::InsufficientLiquidity {
                requested,
                available,
            } => {
                write!(
                    f,
                    "insufficient liquidity: requested {}, available {}",
                    requested, available
                )
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
                requested: Quantity(100),
                available: Quantity(50),
            }
            .to_string(),
            "insufficient liquidity: requested 100, available 50"
        );
        assert_eq!(
            CancelReason::PostOnlyWouldTake.to_string(),
            "post-only order would remove liquidity"
        );
    }
}
