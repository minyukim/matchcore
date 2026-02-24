use std::fmt;

use serde::{Deserialize, Serialize};

/// Reference price type for pegged orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PegReference {
    /// Pegged to the primary price (same side best price)
    Primary,
    /// Pegged to the market price (opposite side best price)
    Market,
    /// Pegged to the mid price between the best bid and the best ask
    MidPrice,
    // TODO: Add last trade price reference
}

impl PegReference {
    pub const COUNT: usize = 3;

    #[inline]
    pub const fn as_index(&self) -> usize {
        match self {
            PegReference::Primary => 0,
            PegReference::Market => 1,
            PegReference::MidPrice => 2,
        }
    }
}

impl fmt::Display for PegReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PegReference::Primary => write!(f, "Primary"),
            PegReference::Market => write!(f, "Market"),
            PegReference::MidPrice => write!(f, "MidPrice"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_index() {
        assert_eq!(PegReference::Primary.as_index(), 0);
        assert_eq!(PegReference::Market.as_index(), 1);
        assert_eq!(PegReference::MidPrice.as_index(), 2);
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&PegReference::Primary).unwrap(),
            "\"Primary\""
        );
        assert_eq!(
            serde_json::to_string(&PegReference::Market).unwrap(),
            "\"Market\""
        );
        assert_eq!(
            serde_json::to_string(&PegReference::MidPrice).unwrap(),
            "\"MidPrice\""
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<PegReference>("\"Primary\"").unwrap(),
            PegReference::Primary
        );
        assert_eq!(
            serde_json::from_str::<PegReference>("\"Market\"").unwrap(),
            PegReference::Market
        );
        assert_eq!(
            serde_json::from_str::<PegReference>("\"MidPrice\"").unwrap(),
            PegReference::MidPrice
        );
    }

    #[test]
    fn test_round_trip_serialization() {
        // Test from_str -> to_string round trip
        for peg_reference in [
            PegReference::Primary,
            PegReference::Market,
            PegReference::MidPrice,
        ] {
            let serialized = serde_json::to_string(&peg_reference).unwrap();
            let deserialized: PegReference = serde_json::from_str(&serialized).unwrap();
            assert_eq!(peg_reference, deserialized);
        }
    }

    #[test]
    fn test_invalid_deserialization() {
        assert!(serde_json::from_str::<PegReference>("\"INVALID\"").is_err());
        assert!(serde_json::from_str::<PegReference>("\"BEST_BID\"").is_err());
        assert!(serde_json::from_str::<PegReference>("\"BEST_ASK\"").is_err());
        assert!(serde_json::from_str::<PegReference>("\"MID_PRICE\"").is_err());
        assert!(serde_json::from_str::<PegReference>("\"LAST_TRADE\"").is_err());
        assert!(serde_json::from_str::<PegReference>("123").is_err());
        assert!(serde_json::from_str::<PegReference>("null").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(PegReference::Primary.to_string(), "Primary");
        assert_eq!(PegReference::Market.to_string(), "Market");
        assert_eq!(PegReference::MidPrice.to_string(), "MidPrice");
    }
}
