use std::fmt;

/// Reference price type for pegged orders
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// The number of peg reference types
    pub const COUNT: usize = 3;

    /// Convert the peg reference to an index
    #[inline]
    pub const fn as_index(self) -> usize {
        match self {
            PegReference::Primary => 0,
            PegReference::Market => 1,
            PegReference::MidPrice => 2,
        }
    }

    /// Whether the peg reference is always a taker
    #[inline]
    pub const fn is_always_taker(self) -> bool {
        matches!(self, PegReference::Market)
    }

    /// Whether the peg reference can be a taker
    #[inline]
    pub const fn can_be_taker(self) -> bool {
        matches!(self, PegReference::Market)
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
    fn test_can_be_taker() {
        assert!(!PegReference::Primary.can_be_taker());
        assert!(PegReference::Market.can_be_taker());
        assert!(!PegReference::MidPrice.can_be_taker());
    }

    #[test]
    fn test_display() {
        assert_eq!(PegReference::Primary.to_string(), "Primary");
        assert_eq!(PegReference::Market.to_string(), "Market");
        assert_eq!(PegReference::MidPrice.to_string(), "MidPrice");
    }

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "serde")]
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
}
