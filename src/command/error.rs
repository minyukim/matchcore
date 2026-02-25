use std::fmt;

use serde::{Deserialize, Serialize};

/// Error that violates the invariants of a command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandError {
    /// The quantity of the order is zero
    ZeroQuantity,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::ZeroQuantity => write!(f, "Quantity is zero"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        {
            assert_eq!(CommandError::ZeroQuantity.to_string(), "Quantity is zero");
        }
    }
}
