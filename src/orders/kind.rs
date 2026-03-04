use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents the kind of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderKind {
    /// Limit order
    Limit,
    /// Pegged order
    Pegged,
}

impl fmt::Display for OrderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderKind::Limit => write!(f, "Limit"),
            OrderKind::Pegged => write!(f, "Pegged"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(OrderKind::Limit.to_string(), "Limit");
        assert_eq!(OrderKind::Pegged.to_string(), "Pegged");
    }
}
