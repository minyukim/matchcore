#[cfg(test)]
mod tests_price_level {
    use crate::{LimitOrder, OrderCore, QuantityPolicy, Side, TimeInForce, book::PriceLevel};

    use std::collections::HashMap;

    #[test]
    fn test_total_quantity() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.total_quantity(), 0);

        price_level.visible_quantity = 10;
        price_level.hidden_quantity = 20;
        assert_eq!(price_level.total_quantity(), 30);
    }

    #[test]
    fn test_order_count() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.order_count(), 0);
        assert!(price_level.is_empty());

        price_level.increment_order_count();
        assert_eq!(price_level.order_count(), 1);
        assert!(!price_level.is_empty());

        price_level.decrement_order_count();
        assert_eq!(price_level.order_count(), 0);
        assert!(price_level.is_empty());
    }

    #[test]
    fn test_push() {
        let mut limit_orders = HashMap::new();
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.visible_quantity, 0);
        assert_eq!(price_level.hidden_quantity, 0);
        assert_eq!(price_level.order_count(), 0);

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Standard { quantity: 10 },
            ),
        );
        assert_eq!(price_level.visible_quantity, 10);
        assert_eq!(price_level.hidden_quantity, 0);
        assert_eq!(price_level.order_count(), 1);

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.visible_quantity, 30);
        assert_eq!(price_level.hidden_quantity, 0);
        assert_eq!(price_level.order_count(), 2);

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(2, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Iceberg {
                    visible_quantity: 10,
                    hidden_quantity: 20,
                    replenish_quantity: 10,
                },
            ),
        );
        assert_eq!(price_level.visible_quantity, 40);
        assert_eq!(price_level.hidden_quantity, 20);
        assert_eq!(price_level.order_count(), 3);
    }

    #[test]
    fn test_peek_order_id() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Standard { quantity: 10 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));
    }

    #[test]
    fn test_peek() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek(&mut limit_orders).is_none());

        let mut order = LimitOrder::new(
            OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
            100,
            QuantityPolicy::Standard { quantity: 10 },
        );
        price_level.push(&mut limit_orders, order.clone());
        assert_eq!(price_level.peek(&mut limit_orders), Some(&mut order));

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.peek(&mut limit_orders), Some(&mut order));
    }

    #[test]
    fn test_remove_head_order() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Standard { quantity: 10 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek_order_id(&limit_orders).is_none());

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(1));

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(2, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Standard { quantity: 30 },
            ),
        );
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(1));

        price_level.remove_head_order(&mut limit_orders);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(2));

        price_level.remove_head_order(&mut limit_orders);
        assert!(price_level.peek_order_id(&limit_orders).is_none());
    }

    #[test]
    fn test_handle_replenishment() {
        let mut limit_orders = HashMap::new();

        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.visible_quantity, 0);
        assert_eq!(price_level.hidden_quantity, 0);
        assert_eq!(price_level.order_count(), 0);
        assert!(price_level.peek(&mut limit_orders).is_none());

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Iceberg {
                    visible_quantity: 0,
                    hidden_quantity: 100,
                    replenish_quantity: 10,
                },
            ),
        );
        assert_eq!(price_level.visible_quantity, 0);
        assert_eq!(price_level.hidden_quantity, 100);
        assert_eq!(price_level.order_count(), 1);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.handle_replenishment(10);
        assert_eq!(price_level.visible_quantity, 10);
        assert_eq!(price_level.hidden_quantity, 90);
        assert_eq!(price_level.order_count(), 1);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.push(
            &mut limit_orders,
            LimitOrder::new(
                OrderCore::new(1, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                100,
                QuantityPolicy::Standard { quantity: 20 },
            ),
        );
        assert_eq!(price_level.visible_quantity, 30);
        assert_eq!(price_level.hidden_quantity, 90);
        assert_eq!(price_level.order_count(), 2);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(0));

        price_level.handle_replenishment(10);
        assert_eq!(price_level.visible_quantity, 40);
        assert_eq!(price_level.hidden_quantity, 80);
        assert_eq!(price_level.order_count(), 2);
        assert_eq!(price_level.peek_order_id(&limit_orders), Some(1));
    }
}
