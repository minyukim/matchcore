use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents the quantity policy of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantityPolicy {
    /// Standard quantity policy
    Standard {
        /// The quantity of the order
        quantity: u64,
    },
    /// Iceberg quantity policy
    Iceberg {
        /// The visible quantity of the order
        visible_quantity: u64,
        /// The hidden quantity of the order
        hidden_quantity: u64,
        /// The replenish quantity of the order
        replenish_quantity: u64,
    },
}

impl QuantityPolicy {
    /// Get the quantity of the order
    pub fn visible_quantity(&self) -> u64 {
        match self {
            QuantityPolicy::Standard { quantity } => *quantity,
            QuantityPolicy::Iceberg {
                visible_quantity, ..
            } => *visible_quantity,
        }
    }

    /// Get the hidden quantity of the order
    pub fn hidden_quantity(&self) -> u64 {
        match self {
            QuantityPolicy::Iceberg {
                hidden_quantity, ..
            } => *hidden_quantity,
            _ => 0,
        }
    }

    /// Get the replenish quantity of the order
    pub fn replenish_quantity(&self) -> u64 {
        match self {
            QuantityPolicy::Iceberg {
                replenish_quantity, ..
            } => *replenish_quantity,
            _ => 0,
        }
    }

    /// Get the total quantity of the order
    pub fn total_quantity(&self) -> u64 {
        self.visible_quantity() + self.hidden_quantity()
    }

    /// Check if the order is filled
    pub fn is_filled(&self) -> bool {
        self.total_quantity() == 0
    }

    /// Update the visible quantity of the order
    pub fn update_visible_quantity(&mut self, new_visible_quantity: u64) {
        match self {
            QuantityPolicy::Standard { quantity } => *quantity = new_visible_quantity,
            QuantityPolicy::Iceberg {
                visible_quantity, ..
            } => *visible_quantity = new_visible_quantity,
        }
    }

    /// Replenish the hidden quantity of the order
    pub fn replenish(&mut self) -> u64 {
        match self {
            QuantityPolicy::Iceberg {
                visible_quantity,
                hidden_quantity,
                replenish_quantity,
            } => {
                let new_hidden = hidden_quantity.saturating_sub(*replenish_quantity);
                let replenished = *hidden_quantity - new_hidden;

                *visible_quantity = visible_quantity.saturating_add(replenished);
                *hidden_quantity = new_hidden;

                replenished
            }
            _ => 0,
        }
    }
}

impl fmt::Display for QuantityPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantityPolicy::Standard { quantity } => write!(f, "Standard: {}", quantity),
            QuantityPolicy::Iceberg {
                visible_quantity,
                hidden_quantity,
                replenish_quantity,
            } => write!(
                f,
                "Iceberg: visible_quantity={} hidden_quantity={} replenish_quantity={}",
                visible_quantity, hidden_quantity, replenish_quantity
            ),
        }
    }
}
