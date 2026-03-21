use std::fmt;

/// Represents a sequence number
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SequenceNumber(pub u64);

impl SequenceNumber {
    /// Get the next sequence number
    pub fn next(&self) -> SequenceNumber {
        SequenceNumber(self.0 + 1)
    }
}

impl fmt::Display for SequenceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next() {
        assert_eq!(SequenceNumber(0).next(), SequenceNumber(1));
        assert_eq!(SequenceNumber(1).next(), SequenceNumber(2));
        assert_eq!(SequenceNumber(2).next(), SequenceNumber(3));
    }

    #[test]
    fn test_display() {
        assert_eq!(SequenceNumber(0).to_string(), "0");
        assert_eq!(SequenceNumber(1).to_string(), "1");
        assert_eq!(SequenceNumber(2).to_string(), "2");
    }
}
