use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents the type of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// Limit order
    Limit,
    /// Pegged order
    Pegged,
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderType::Limit => write!(f, "Limit"),
            OrderType::Pegged => write!(f, "Pegged"),
        }
    }
}
