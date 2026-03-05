use crate::{CommandError, Quantity};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Reason for rejecting a command that cannot be executed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectReason {
    /// The command is invalid
    CommandError(CommandError),
    /// The order was not found
    OrderNotFound,
}

impl fmt::Display for RejectReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RejectReason::CommandError(e) => write!(f, "Command error: {e}"),
            RejectReason::OrderNotFound => write!(f, "Order not found"),
        }
    }
}

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
                write!(f, "Insufficient liquidity: available={}", available)
            }
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
        assert_eq!(RejectReason::OrderNotFound.to_string(), "Order not found");
    }

    #[test]
    fn test_display_cancel_reason() {
        assert_eq!(
            CancelReason::InsufficientLiquidity {
                available: Quantity(50),
            }
            .to_string(),
            "Insufficient liquidity: available=50"
        );
        assert_eq!(
            CancelReason::PostOnlyWouldTake.to_string(),
            "Post-only order would remove liquidity"
        );
    }
}
