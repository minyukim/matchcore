#[cfg(test)]
mod tests_order_processing_result {
    use crate::{
        Side,
        execution::{CancelReason, MatchResult, OrderProcessingResult},
    };

    fn create_order_processing_result() -> OrderProcessingResult {
        OrderProcessingResult::new(1)
    }

    #[test]
    fn test_order_id() {
        assert_eq!(create_order_processing_result().order_id(), 1);
    }

    #[test]
    fn test_cancel_reason() {
        let mut order_processing_result = create_order_processing_result();
        assert_eq!(order_processing_result.cancel_reason(), None);

        let cancel_reason = CancelReason::InsufficientLiquidity {
            requested_quantity: 100,
            available_quantity: 50,
        };
        order_processing_result.set_cancel_reason(cancel_reason.clone());
        assert_eq!(
            order_processing_result.cancel_reason(),
            Some(&cancel_reason)
        );
    }

    #[test]
    fn test_match_result() {
        let mut order_processing_result = create_order_processing_result();
        assert!(order_processing_result.match_result().is_none());

        let match_result = MatchResult::new(Side::Buy);
        order_processing_result.set_match_result(match_result);
        assert!(order_processing_result.match_result().is_some());
    }
}
