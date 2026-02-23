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
