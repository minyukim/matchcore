#[cfg(test)]
mod tests_peg_level {
    use crate::book::PegLevel;

    #[test]
    fn test_push_and_peek() {
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.peek(), None);

        peg_level.push(1);
        assert_eq!(peg_level.peek(), Some(1));
        assert_eq!(peg_level.peek(), Some(1));

        peg_level.push(2);
        assert_eq!(peg_level.peek(), Some(1));
    }

    #[test]
    fn test_push_and_pop() {
        let mut peg_level = PegLevel::new();
        assert_eq!(peg_level.pop(), None);

        peg_level.push(1);
        assert_eq!(peg_level.pop(), Some(1));
        assert_eq!(peg_level.pop(), None);

        peg_level.push(2);
        peg_level.push(3);
        assert_eq!(peg_level.pop(), Some(2));
        assert_eq!(peg_level.pop(), Some(3));
        assert_eq!(peg_level.pop(), None);
    }
}
