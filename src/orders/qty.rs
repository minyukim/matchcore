use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents the quantity policy of an order
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QtyPolicy {
    /// Standard quantity policy
    Standard {
        /// The quantity of the order
        qty: u64,
    },
    /// Iceberg quantity policy
    Iceberg {
        /// The visible quantity of the order
        visible: u64,
        /// The hidden quantity of the order
        hidden: u64,
        /// The replenish amount of the order
        replenish: u64,
    },
}

impl QtyPolicy {
    /// Get the quantity of the order
    pub fn visible_qty(&self) -> u64 {
        match self {
            QtyPolicy::Standard { qty } => *qty,
            QtyPolicy::Iceberg { visible, .. } => *visible,
        }
    }

    /// Get the hidden quantity of the order
    pub fn hidden_qty(&self) -> u64 {
        match self {
            QtyPolicy::Iceberg { hidden, .. } => *hidden,
            _ => 0,
        }
    }

    /// Get the replenish amount of the order
    pub fn replenish_amount(&self) -> u64 {
        match self {
            QtyPolicy::Iceberg { replenish, .. } => *replenish,
            _ => 0,
        }
    }
}

impl fmt::Display for QtyPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QtyPolicy::Standard { qty } => write!(f, "Standard: {}", qty),
            QtyPolicy::Iceberg {
                visible,
                hidden,
                replenish,
            } => write!(
                f,
                "Iceberg: visible={}, hidden={}, replenish={}",
                visible, hidden, replenish
            ),
        }
    }
}
