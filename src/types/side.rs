use std::fmt;

/// Represents the side of an order
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Buy side (bids)
    #[cfg_attr(feature = "serde", serde(rename = "BUY"))]
    Buy,
    /// Sell side (asks)
    #[cfg_attr(feature = "serde", serde(rename = "SELL"))]
    Sell,
}

impl Side {
    /// Returns the opposite side of the order.
    ///
    /// # Examples
    ///
    /// ```
    /// use matchcore::Side;
    /// let buy_side = Side::Buy;
    /// let sell_side = buy_side.opposite();
    /// assert_eq!(sell_side, Side::Sell);
    ///
    /// let sell_side = Side::Sell;
    /// let buy_side = sell_side.opposite();
    /// assert_eq!(buy_side, Side::Buy);
    /// ```
    pub fn opposite(&self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_equality() {
        assert_eq!(Side::Buy, Side::Buy);
        assert_eq!(Side::Sell, Side::Sell);
        assert_ne!(Side::Buy, Side::Sell);
    }

    #[test]
    fn test_side_clone() {
        let buy = Side::Buy;
        let cloned_buy = buy;
        assert_eq!(buy, cloned_buy);

        let sell = Side::Sell;
        let cloned_sell = sell;
        assert_eq!(sell, cloned_sell);
    }

    #[test]
    fn test_display() {
        assert_eq!(Side::Buy.to_string(), "BUY");
        assert_eq!(Side::Sell.to_string(), "SELL");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serialize() {
        assert_eq!(serde_json::to_string(&Side::Buy).unwrap(), "\"BUY\"");
        assert_eq!(serde_json::to_string(&Side::Sell).unwrap(), "\"SELL\"");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize() {
        assert_eq!(serde_json::from_str::<Side>("\"BUY\"").unwrap(), Side::Buy);
        assert_eq!(
            serde_json::from_str::<Side>("\"SELL\"").unwrap(),
            Side::Sell
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_round_trip_serialization() {
        for side in [Side::Buy, Side::Sell] {
            let serialized = serde_json::to_string(&side).unwrap();
            let deserialized: Side = serde_json::from_str(&serialized).unwrap();
            assert_eq!(side, deserialized);
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_invalid_deserialization() {
        assert!(serde_json::from_str::<Side>("\"INVALID\"").is_err());
        assert!(serde_json::from_str::<Side>("\"BUYING\"").is_err());
        assert!(serde_json::from_str::<Side>("\"SELLING\"").is_err());
        assert!(serde_json::from_str::<Side>("123").is_err());
        assert!(serde_json::from_str::<Side>("null").is_err());
    }
}
