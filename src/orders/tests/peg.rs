#[cfg(test)]
mod tests_peg_reference {
    use crate::orders::PegReference;

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&PegReference::BestBid).unwrap(),
            "\"BestBid\""
        );
        assert_eq!(
            serde_json::to_string(&PegReference::BestAsk).unwrap(),
            "\"BestAsk\""
        );
        assert_eq!(
            serde_json::to_string(&PegReference::MidPrice).unwrap(),
            "\"MidPrice\""
        );
        assert_eq!(
            serde_json::to_string(&PegReference::LastTrade).unwrap(),
            "\"LastTrade\""
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<PegReference>("\"BestBid\"").unwrap(),
            PegReference::BestBid
        );
        assert_eq!(
            serde_json::from_str::<PegReference>("\"BestAsk\"").unwrap(),
            PegReference::BestAsk
        );
        assert_eq!(
            serde_json::from_str::<PegReference>("\"MidPrice\"").unwrap(),
            PegReference::MidPrice
        );
        assert_eq!(
            serde_json::from_str::<PegReference>("\"LastTrade\"").unwrap(),
            PegReference::LastTrade
        );
    }

    #[test]
    fn test_round_trip_serialization() {
        // Test from_str -> to_string round trip
        for reference in [
            PegReference::BestBid,
            PegReference::BestAsk,
            PegReference::MidPrice,
            PegReference::LastTrade,
        ] {
            let serialized = serde_json::to_string(&reference).unwrap();
            let deserialized: PegReference = serde_json::from_str(&serialized).unwrap();
            assert_eq!(reference, deserialized);
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
        assert_eq!(PegReference::BestBid.to_string(), "BestBid");
        assert_eq!(PegReference::BestAsk.to_string(), "BestAsk");
        assert_eq!(PegReference::MidPrice.to_string(), "MidPrice");
        assert_eq!(PegReference::LastTrade.to_string(), "LastTrade");
    }
}
