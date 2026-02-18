#[cfg(test)]
mod tests_price_level {
    use crate::book::PriceLevel;

    #[test]
    fn test_total_quantity() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.total_quantity(), 0);

        price_level.visible_quantity = 10;
        price_level.hidden_quantity = 20;
        assert_eq!(price_level.total_quantity(), 30);
    }

    #[test]
    fn test_push_and_peek() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.peek(), None);

        price_level.push(1);
        assert_eq!(price_level.peek(), Some(&1));
        assert_eq!(price_level.peek(), Some(&1));

        price_level.push(2);
        assert_eq!(price_level.peek(), Some(&1));
    }

    #[test]
    fn test_push_and_pop() {
        let mut price_level = PriceLevel::new();
        assert_eq!(price_level.pop(), None);

        price_level.push(1);
        assert_eq!(price_level.pop(), Some(1));
        assert_eq!(price_level.pop(), None);

        price_level.push(2);
        price_level.push(3);
        assert_eq!(price_level.pop(), Some(2));
        assert_eq!(price_level.pop(), Some(3));
        assert_eq!(price_level.pop(), None);
    }
}
