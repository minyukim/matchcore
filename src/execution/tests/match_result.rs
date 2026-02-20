#[cfg(test)]
mod tests_match_result {
    use crate::{
        Side,
        execution::{CancelReason, MatchResult, Trade},
    };

    fn create_match_result() -> MatchResult {
        MatchResult::new(1, Side::Buy)
    }

    #[test]
    fn test_taker_order_id() {
        assert_eq!(create_match_result().taker_order_id(), 1);
    }

    #[test]
    fn test_taker_side() {
        assert_eq!(create_match_result().taker_side(), Side::Buy);
    }

    #[test]
    fn test_executed_quantity() {
        assert_eq!(create_match_result().executed_quantity(), 0);
    }

    #[test]
    fn test_executed_value() {
        assert_eq!(create_match_result().executed_value(), 0);
    }

    #[test]
    fn test_cancel_reason() {
        let mut match_result = create_match_result();
        assert_eq!(match_result.cancel_reason(), None);

        let reason = CancelReason::InsufficientLiquidity {
            requested_quantity: 100,
            available_quantity: 50,
        };
        match_result.set_cancel_reason(reason.clone());
        assert_eq!(match_result.cancel_reason(), Some(&reason));
    }

    #[test]
    fn test_trades() {
        let mut match_result = create_match_result();
        assert_eq!(match_result.trades(), &[]);

        let trades = [
            Trade::new(2, 99, 20),
            Trade::new(3, 100, 30),
            Trade::new(4, 101, 20),
        ];
        let expected_executed_quantities = [20, 50, 70];
        let expected_executed_values = [1980, 4980, 7000];

        for (i, trade) in trades.iter().enumerate() {
            match_result.add_trade(*trade);
            assert_eq!(
                match_result.executed_quantity(),
                expected_executed_quantities[i]
            );
            assert_eq!(match_result.executed_value(), expected_executed_values[i]);
        }
        assert_eq!(match_result.trades(), &trades);
    }

    #[test]
    fn test_expired_order_ids() {
        let mut match_result = create_match_result();
        assert_eq!(match_result.expired_order_ids(), &Vec::<u64>::new());

        match_result.add_expired_order_id(4);
        assert_eq!(match_result.expired_order_ids(), &[4]);

        match_result.add_expired_order_id(5);
        assert_eq!(match_result.expired_order_ids(), &[4, 5]);
    }
}

#[cfg(test)]
mod tests_trade {
    use crate::execution::Trade;

    fn create_trade() -> Trade {
        Trade::new(1, 100, 10)
    }

    #[test]
    fn test_maker_order_id() {
        assert_eq!(create_trade().maker_order_id(), 1);
    }

    #[test]
    fn test_price() {
        assert_eq!(create_trade().price(), 100);
    }

    #[test]
    fn test_quantity() {
        assert_eq!(create_trade().quantity(), 10);
    }

    #[test]
    fn test_round_trip_serialization() {
        let trade = create_trade();
        let serialized = serde_json::to_string(&trade).unwrap();
        let deserialized: Trade = serde_json::from_str(&serialized).unwrap();
        assert_eq!(trade, deserialized);
    }

    #[test]
    fn test_display() {
        assert_eq!(
            create_trade().to_string(),
            "Trade: maker_order_id=1 price=100 quantity=10"
        );
    }
}
