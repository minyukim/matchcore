use super::SequenceNumber;

use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents an order ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OrderId(pub u64);

impl From<SequenceNumber> for OrderId {
    fn from(sequence_number: SequenceNumber) -> Self {
        OrderId(sequence_number.0)
    }
}

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_sequence_number() {
        assert_eq!(OrderId::from(SequenceNumber(1)), OrderId(1));
        assert_eq!(OrderId::from(SequenceNumber(2)), OrderId(2));
        assert_eq!(OrderId::from(SequenceNumber(3)), OrderId(3));
    }

    #[test]
    fn test_display() {
        assert_eq!(OrderId(1).to_string(), "1");
        assert_eq!(OrderId(2).to_string(), "2");
        assert_eq!(OrderId(3).to_string(), "3");
    }
}
