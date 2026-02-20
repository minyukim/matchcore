#[cfg(test)]
mod tests_cancel_reason {
    use crate::execution::CancelReason;

    #[test]
    fn test_display() {
        let reason = CancelReason::InsufficientLiquidity {
            requested_quantity: 100,
            available_quantity: 50,
        };
        assert_eq!(
            reason.to_string(),
            "Insufficient liquidity: requested=100 available=50"
        );
    }
}
