use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents the quantity policy of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QtyPolicy {
    /// Standard quantity policy
    Standard {
        /// The quantity of the order
        qty: u64,
    },
    /// Iceberg quantity policy
    Iceberg {
        /// The visible quantity of the order
        visible_qty: u64,
        /// The hidden quantity of the order
        hidden_qty: u64,
        /// The replenish size of the order
        replenish_size: u64,
    },
}

impl QtyPolicy {
    /// Get the quantity of the order
    pub fn visible_qty(&self) -> u64 {
        match self {
            QtyPolicy::Standard { qty } => *qty,
            QtyPolicy::Iceberg { visible_qty, .. } => *visible_qty,
        }
    }

    /// Get the hidden quantity of the order
    pub fn hidden_qty(&self) -> u64 {
        match self {
            QtyPolicy::Iceberg { hidden_qty, .. } => *hidden_qty,
            _ => 0,
        }
    }

    /// Get the replenish size of the order
    pub fn replenish_size(&self) -> u64 {
        match self {
            QtyPolicy::Iceberg { replenish_size, .. } => *replenish_size,
            _ => 0,
        }
    }

    pub fn update_visible_qty(&mut self, new_visible_qty: u64) {
        match self {
            QtyPolicy::Standard { qty } => *qty = new_visible_qty,
            QtyPolicy::Iceberg { visible_qty, .. } => *visible_qty = new_visible_qty,
        }
    }

    pub fn replenish(&mut self) -> u64 {
        match self {
            QtyPolicy::Iceberg {
                visible_qty,
                hidden_qty,
                replenish_size,
            } => {
                let new_hidden = hidden_qty.saturating_sub(*replenish_size);
                let replenished = *hidden_qty - new_hidden;

                *visible_qty = visible_qty.saturating_add(replenished);
                *hidden_qty = new_hidden;

                replenished
            }
            _ => 0,
        }
    }
}

impl fmt::Display for QtyPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QtyPolicy::Standard { qty } => write!(f, "Standard: {}", qty),
            QtyPolicy::Iceberg {
                visible_qty,
                hidden_qty,
                replenish_size,
            } => write!(
                f,
                "Iceberg: visible_qty={} hidden_qty={} replenish_size={}",
                visible_qty, hidden_qty, replenish_size
            ),
        }
    }
}
