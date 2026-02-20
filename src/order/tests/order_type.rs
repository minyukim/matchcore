#[cfg(test)]
mod tests_order_type {
    use crate::order::OrderType;

    #[test]
    fn test_round_trip_serialization() {
        for order_type in [OrderType::Limit, OrderType::Pegged] {
            let serialized = serde_json::to_string(&order_type).unwrap();
            let deserialized: OrderType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(order_type, deserialized);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(OrderType::Limit.to_string(), "Limit");
        assert_eq!(OrderType::Pegged.to_string(), "Pegged");
    }
}
