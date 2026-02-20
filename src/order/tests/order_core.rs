#[cfg(test)]
mod tests_order_core {
    use crate::order::{OrderCore, Side, TimeInForce};

    fn create_order_core() -> OrderCore {
        OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ())
    }

    #[test]
    fn test_id() {
        assert_eq!(create_order_core().id(), 0);
    }

    #[test]
    fn test_side() {
        assert_eq!(create_order_core().side(), Side::Buy);
    }

    #[test]
    fn test_is_post_only() {
        assert!(create_order_core().is_post_only());
    }

    #[test]
    fn test_timestamp() {
        assert_eq!(create_order_core().timestamp(), 1771180000);
    }

    #[test]
    fn test_time_in_force() {
        let mut order = create_order_core();
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
    fn test_roundtrip_serialization() {
        let order = create_order_core();
        let serialized = serde_json::to_string(&order).unwrap();
        let deserialized: OrderCore = serde_json::from_str(&serialized).unwrap();
        assert_eq!(order, deserialized);
    }
}
