use std::fmt;

use serde::{Deserialize, Serialize};

/// Error that violates the invariants of a command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandError {
    /// The price of the order is zero
    ZeroPrice,
    /// The quantity of the order is zero
    ZeroQuantity,
    /// The hidden quantity of the iceberg order is zero
    IcebergZeroHiddenQuantity,
    /// The replenish quantity of the iceberg order is zero
    IcebergZeroReplenishQuantity,
    /// The iceberg order has an immediate time in force
    IcebergImmediateTif,
    /// The order is post-only but has an immediate time in force
    PostOnlyImmediateTif,
    /// The pegged order cannot be a taker but has an immediate time in force
    PeggedNonTakerImmediateTif,
    /// The pegged order is always a taker but is post-only
    PeggedAlwaysTakerPostOnly,
    /// The patch is empty
    EmptyPatch,
    /// The command has expired
    Expired,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::ZeroPrice => write!(f, "Price is zero"),
            CommandError::ZeroQuantity => write!(f, "Quantity is zero"),
            CommandError::IcebergZeroHiddenQuantity => write!(f, "Iceberg hidden quantity is zero"),
            CommandError::IcebergZeroReplenishQuantity => {
                write!(f, "Iceberg replenish quantity is zero")
            }
            CommandError::IcebergImmediateTif => {
                write!(f, "Iceberg order has an immediate time in force")
            }
            CommandError::PostOnlyImmediateTif => {
                write!(f, "Order is post-only but has an immediate time in force")
            }
            CommandError::PeggedNonTakerImmediateTif => {
                write!(
                    f,
                    "Pegged order cannot be a taker but has an immediate time in force"
                )
            }
            CommandError::PeggedAlwaysTakerPostOnly => {
                write!(f, "Pegged order is always a taker but is post-only")
            }
            CommandError::EmptyPatch => write!(f, "Patch is empty"),
            CommandError::Expired => write!(f, "Command has expired"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(CommandError::ZeroPrice.to_string(), "Price is zero");
        assert_eq!(CommandError::ZeroQuantity.to_string(), "Quantity is zero");
        assert_eq!(
            CommandError::IcebergZeroHiddenQuantity.to_string(),
            "Iceberg hidden quantity is zero"
        );
        assert_eq!(
            CommandError::IcebergZeroReplenishQuantity.to_string(),
            "Iceberg replenish quantity is zero"
        );
        assert_eq!(
            CommandError::IcebergImmediateTif.to_string(),
            "Iceberg order has an immediate time in force"
        );
        assert_eq!(
            CommandError::PostOnlyImmediateTif.to_string(),
            "Order is post-only but has an immediate time in force"
        );
        assert_eq!(
            CommandError::PeggedNonTakerImmediateTif.to_string(),
            "Pegged order cannot be a taker but has an immediate time in force"
        );
        assert_eq!(CommandError::EmptyPatch.to_string(), "Patch is empty");
        assert_eq!(CommandError::Expired.to_string(), "Command has expired");
    }
}
