use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents a timestamp
/// The timestamp is expressed as a Unix timestamp (seconds since epoch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(Timestamp(0).to_string(), "0");
        assert_eq!(Timestamp(1).to_string(), "1");
        assert_eq!(Timestamp(2).to_string(), "2");
    }
}
