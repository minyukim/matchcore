#[cfg(test)]
mod tests_peg_level {
    use crate::{OrderCore, PegReference, PeggedOrder, Side, TimeInForce, book::PegLevel};

    use std::collections::HashMap;

    #[test]
    fn test_order_count() {
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.order_count(), 0);

        peg_level.increment_order_count();
        assert_eq!(peg_level.order_count(), 1);

        peg_level.decrement_order_count();
        assert_eq!(peg_level.order_count(), 0);
    }

    #[test]
    fn test_push() {
        let mut limit_orders = HashMap::new();
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.quantity, 0);
        assert_eq!(peg_level.order_count(), 0);

        peg_level.push(
            &mut limit_orders,
            PeggedOrder::new(
                OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                PegReference::Primary,
                10,
            ),
        );
        assert_eq!(peg_level.quantity, 10);
        assert_eq!(peg_level.order_count(), 1);

        peg_level.push(
            &mut limit_orders,
            PeggedOrder::new(
                OrderCore::new(1, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                PegReference::Primary,
                20,
            ),
        );
        assert_eq!(peg_level.quantity, 30);
        assert_eq!(peg_level.order_count(), 2);

        peg_level.push(
            &mut limit_orders,
            PeggedOrder::new(
                OrderCore::new(2, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                PegReference::Primary,
                30,
            ),
        );
        assert_eq!(peg_level.quantity, 60);
        assert_eq!(peg_level.order_count(), 3);
    }

    #[test]
    fn test_peek_order_id() {
        let mut pegged_orders = HashMap::new();

        let mut peg_level = PegLevel::new();
        assert!(peg_level.peek_order_id(&pegged_orders).is_none());

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                PegReference::Primary,
                100,
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(0));

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderCore::new(1, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                PegReference::Primary,
                100,
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(0));
    }

    #[test]
    fn test_peek() {
        let mut pegged_orders = HashMap::new();

        let mut peg_level = PegLevel::new();
        assert!(peg_level.peek(&mut pegged_orders).is_none());

        let mut order = PeggedOrder::new(
            OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
            PegReference::Primary,
            100,
        );
        peg_level.push(&mut pegged_orders, order.clone());
        assert_eq!(peg_level.peek(&mut pegged_orders), Some(&mut order));

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderCore::new(1, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                PegReference::Primary,
                100,
            ),
        );
        assert_eq!(peg_level.peek(&mut pegged_orders), Some(&mut order));
    }

    #[test]
    fn test_remove_head_order() {
        let mut pegged_orders = HashMap::new();

        let mut peg_level = PegLevel::new();
        assert!(peg_level.peek_order_id(&pegged_orders).is_none());

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderCore::new(0, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                PegReference::Primary,
                100,
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(0));

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek_order_id(&pegged_orders).is_none());

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderCore::new(1, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                PegReference::Primary,
                100,
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(1));

        peg_level.push(
            &mut pegged_orders,
            PeggedOrder::new(
                OrderCore::new(2, Side::Buy, true, 1771180000, TimeInForce::Gtc, ()),
                PegReference::Primary,
                100,
            ),
        );
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(1));

        peg_level.remove_head_order(&mut pegged_orders);
        assert_eq!(peg_level.peek_order_id(&pegged_orders), Some(2));

        peg_level.remove_head_order(&mut pegged_orders);
        assert!(peg_level.peek_order_id(&pegged_orders).is_none());
    }
}
