#[cfg(test)]
mod tests_pegged_order {
    use crate::order::{OrderCore, PegReference, PeggedOrder, Side, TimeInForce};

    fn create_pegged_order() -> PeggedOrder {
        PeggedOrder::new(
            OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
            PegReference::Primary,
            20,
        )
    }

    #[test]
    fn test_id() {
        assert_eq!(create_pegged_order().id(), 0);
    }

    #[test]
    fn test_peg_reference() {
        let mut order = create_pegged_order();
        assert_eq!(order.peg_reference(), PegReference::Primary);

        order.update_peg_reference(PegReference::Market);
        assert_eq!(order.peg_reference(), PegReference::Market);

        order.update_peg_reference(PegReference::MidPrice);
        assert_eq!(order.peg_reference(), PegReference::MidPrice);

        order.update_peg_reference(PegReference::Primary);
        assert_eq!(order.peg_reference(), PegReference::Primary);
    }

    #[test]
    fn test_quantity() {
        let mut order = create_pegged_order();
        assert_eq!(order.quantity(), 20);
        assert!(!order.is_filled());

        order.update_quantity(30);
        assert_eq!(order.quantity(), 30);
        assert!(!order.is_filled());

        order.update_quantity(10);
        assert_eq!(order.quantity(), 10);
        assert!(!order.is_filled());

        order.update_quantity(0);
        assert_eq!(order.quantity(), 0);
        assert!(order.is_filled());
    }

    #[test]
    fn test_side() {
        assert_eq!(create_pegged_order().side(), Side::Buy);
    }

    #[test]
    fn test_is_post_only() {
        assert!(create_pegged_order().is_post_only());
    }

    #[test]
    fn test_timestamp() {
        assert_eq!(create_pegged_order().timestamp(), 1771180000);
    }

    #[test]
    fn test_time_in_force() {
        let mut order = create_pegged_order();
        assert_eq!(order.time_in_force(), TimeInForce::Gtc);
        assert!(!order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(1771180000));

        order.update_time_in_force(TimeInForce::Ioc);
        assert!(order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(1771180000));

        order.update_time_in_force(TimeInForce::Fok);
        assert!(order.is_immediate());
        assert!(!order.has_expiry());
        assert!(!order.is_expired(1771180000));

        order.update_time_in_force(TimeInForce::Gtd(1771180000 + 1000));
        assert!(!order.is_immediate());
        assert!(order.has_expiry());
        assert!(!order.is_expired(1771180000));
        assert!(order.is_expired(1771180000 + 1000));
    }

    #[test]
    fn test_match_against() {
        let mut order = create_pegged_order();
        assert_eq!(order.quantity(), 20);

        let consumed = order.match_against(2);
        assert_eq!(consumed, 2);
        assert_eq!(order.quantity(), 18);

        let consumed = order.match_against(20);
        assert_eq!(consumed, 18);
        assert_eq!(order.quantity(), 0);

        let consumed = order.match_against(10);
        assert_eq!(consumed, 0);
        assert_eq!(order.quantity(), 0);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let order = create_pegged_order();
        let serialized = serde_json::to_string(&order).unwrap();
        let deserialized: PeggedOrder = serde_json::from_str(&serialized).unwrap();
        assert_eq!(order, deserialized);
    }

    #[test]
    fn test_display() {
        assert_eq!(
            create_pegged_order().to_string(),
            "Pegged: id=0 peg_reference=Primary quantity=20 side=BUY post_only=true timestamp=1771180000 time_in_force=GTC"
        );
    }
}

#[cfg(test)]
mod tests_peg_reference {
    use crate::order::PegReference;

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
