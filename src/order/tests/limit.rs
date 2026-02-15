#[cfg(test)]
mod tests_order {
    use crate::order::{Order, QuantityPolicy, Side, TimeInForce};

    fn create_standard_order() -> Order {
        Order::new(
            0,
            90,
            QuantityPolicy::Standard { quantity: 10 },
            Side::Buy,
            true,
            1771180000,
            TimeInForce::Gtc,
            (),
        )
    }

    fn create_iceberg_order() -> Order {
        Order::new(
            1,
            100,
            QuantityPolicy::Iceberg {
                visible_quantity: 20,
                hidden_quantity: 40,
                replenish_quantity: 20,
            },
            Side::Sell,
            false,
            1771190000,
            TimeInForce::Gtc,
            (),
        )
    }

    #[test]
    fn test_id() {
        assert_eq!(create_standard_order().id(), 0);
        assert_eq!(create_iceberg_order().id(), 1);
    }

    #[test]
    fn test_price() {
        let mut order = create_standard_order();
        assert_eq!(order.price(), 90);

        order.update_price(95);
        assert_eq!(order.price(), 95);

        assert_eq!(create_iceberg_order().price(), 100);
    }

    #[test]
    fn test_quantity() {
        {
            let mut order = create_standard_order();
            assert_eq!(order.visible_quantity(), 10);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 0);

            order.update_quantity(QuantityPolicy::Iceberg {
                visible_quantity: 1,
                hidden_quantity: 10,
                replenish_quantity: 1,
            });

            assert_eq!(order.visible_quantity(), 1);
            assert_eq!(order.hidden_quantity(), 10);
            assert_eq!(order.replenish_quantity(), 1);
        }
        {
            let mut order = create_iceberg_order();
            assert_eq!(order.visible_quantity(), 20);
            assert_eq!(order.hidden_quantity(), 40);
            assert_eq!(order.replenish_quantity(), 20);

            order.update_quantity(QuantityPolicy::Standard { quantity: 100 });
            assert_eq!(order.visible_quantity(), 100);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 0);
        }
    }

    #[test]
    fn test_side() {
        assert_eq!(create_standard_order().side(), Side::Buy);
        assert_eq!(create_iceberg_order().side(), Side::Sell);
    }

    #[test]
    fn test_is_post_only() {
        assert!(create_standard_order().is_post_only());
        assert!(!create_iceberg_order().is_post_only());
    }

    #[test]
    fn test_timestamp() {
        assert_eq!(create_standard_order().timestamp(), 1771180000);
        assert_eq!(create_iceberg_order().timestamp(), 1771190000);
    }

    #[test]
    fn test_time_in_force() {
        let mut order = create_standard_order();
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
        {
            let mut order = create_standard_order();
            assert_eq!(order.visible_quantity(), 10);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 0);

            let (consumed, remaining, replenished) = order.match_against(2);
            assert_eq!(consumed, 2);
            assert_eq!(remaining, 0);
            assert_eq!(replenished, 0);
            assert_eq!(order.visible_quantity(), 8);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 0);

            let (consumed, remaining, replenished) = order.match_against(10);
            assert_eq!(consumed, 8);
            assert_eq!(remaining, 2);
            assert_eq!(replenished, 0);
            assert_eq!(order.visible_quantity(), 0);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 0);

            let (consumed, remaining, replenished) = order.match_against(10);
            assert_eq!(consumed, 0);
            assert_eq!(remaining, 10);
            assert_eq!(replenished, 0);
            assert_eq!(order.visible_quantity(), 0);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 0);
        }
        {
            let mut order = create_iceberg_order();
            assert_eq!(order.visible_quantity(), 20);
            assert_eq!(order.hidden_quantity(), 40);
            assert_eq!(order.replenish_quantity(), 20);

            let (consumed, remaining, replenished) = order.match_against(5);
            assert_eq!(consumed, 5);
            assert_eq!(remaining, 0);
            assert_eq!(replenished, 0);
            assert_eq!(order.visible_quantity(), 15);
            assert_eq!(order.hidden_quantity(), 40);
            assert_eq!(order.replenish_quantity(), 20);

            let (consumed, remaining, replenished) = order.match_against(20);
            assert_eq!(consumed, 15);
            assert_eq!(remaining, 5);
            assert_eq!(replenished, 20);
            assert_eq!(order.visible_quantity(), 20);
            assert_eq!(order.hidden_quantity(), 20);
            assert_eq!(order.replenish_quantity(), 20);

            let (consumed, remaining, replenished) = order.match_against(20);
            assert_eq!(consumed, 20);
            assert_eq!(remaining, 0);
            assert_eq!(replenished, 20);
            assert_eq!(order.visible_quantity(), 20);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 20);

            let (consumed, remaining, replenished) = order.match_against(1);
            assert_eq!(consumed, 1);
            assert_eq!(remaining, 0);
            assert_eq!(replenished, 0);
            assert_eq!(order.visible_quantity(), 19);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 20);

            let (consumed, remaining, replenished) = order.match_against(19);
            assert_eq!(consumed, 19);
            assert_eq!(remaining, 0);
            assert_eq!(replenished, 0);
            assert_eq!(order.visible_quantity(), 0);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 20);

            let (consumed, remaining, replenished) = order.match_against(1);
            assert_eq!(consumed, 0);
            assert_eq!(remaining, 1);
            assert_eq!(replenished, 0);
            assert_eq!(order.visible_quantity(), 0);
            assert_eq!(order.hidden_quantity(), 0);
            assert_eq!(order.replenish_quantity(), 20);
        }
    }

    #[test]
    fn test_roundtrip_serialization() {
        for order in [create_standard_order(), create_iceberg_order()] {
            let serialized = serde_json::to_string(&order).unwrap();
            let deserialized: Order = serde_json::from_str(&serialized).unwrap();
            assert_eq!(order, deserialized);
        }
    }

    #[test]
    fn test_display() {
        {
            assert_eq!(
                create_standard_order().to_string(),
                "Standard: id=0 price=90 quantity=10 side=BUY post_only=true timestamp=1771180000 time_in_force=GTC"
            );
        }
        {
            assert_eq!(
                create_iceberg_order().to_string(),
                "Iceberg: id=1 price=100 visible_quantity=20 hidden_quantity=40 replenish_quantity=20 side=SELL post_only=false timestamp=1771190000 time_in_force=GTC"
            );
        }
    }
}
