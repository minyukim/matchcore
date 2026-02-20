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
        }
    }
}
