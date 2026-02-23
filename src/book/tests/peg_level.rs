#[cfg(test)]
mod tests_peg_level {
    use crate::book::PegLevel;

    #[test]
    fn test_order_count() {
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.order_count(), 0);

        peg_level.increment_order_count();
        assert_eq!(peg_level.order_count(), 1);

        peg_level.decrement_order_count();
        assert_eq!(peg_level.order_count(), 0);
    }
}
