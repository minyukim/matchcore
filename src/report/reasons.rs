use crate::CommandError;

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectReason {
    /// The command is invalid
    CommandError(CommandError),
    /// No liquidity available to fill the immediate order
    NoLiquidity,
    /// The post-only order would remove liquidity
    PostOnlyWouldTake,
}

impl fmt::Display for RejectReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RejectReason::CommandError(e) => write!(f, "Command error: {e}"),
            RejectReason::NoLiquidity => {
                write!(f, "No liquidity available to fill the immediate order")
            }
            RejectReason::PostOnlyWouldTake => {
                write!(f, "Post-only order would remove liquidity")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancelReason {
    /// Insufficient liquidity for market orders and IOC/FOK orders
    InsufficientLiquidity {
        /// The quantity of the order that was requested to be filled
        requested_quantity: u64,
        /// The quantity of the order that was available to be filled
        available_quantity: u64,
    },
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
    fn test_display_reject_reason() {
        assert_eq!(
            RejectReason::CommandError(CommandError::ZeroPrice).to_string(),
            "Command error: Price is zero"
        );
        assert_eq!(
            RejectReason::NoLiquidity.to_string(),
            "No liquidity available to fill the immediate order"
        );
    }

    #[test]
    fn test_display_cancel_reason() {
        assert_eq!(
            CancelReason::InsufficientLiquidity {
                requested_quantity: 100,
                available_quantity: 50,
            }
            .to_string(),
            "Insufficient liquidity: requested=100 available=50"
        );
        assert_eq!(
            CancelReason::PostOnlyWouldTake.to_string(),
            "Post-only order would remove liquidity"
        );
    }
}
