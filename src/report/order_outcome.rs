use crate::{CancelReason, MatchResult, OrderId};

use serde::{Deserialize, Serialize};

/// Outcome of the order execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderOutcome {
    /// The ID of the order
    order_id: OrderId,
    /// The match result if the order was matched
    match_result: Option<MatchResult>,
    /// The reason the order was cancelled, if it was cancelled
    cancel_reason: Option<CancelReason>,
}

impl OrderOutcome {
    /// Create a new order outcome
    pub(crate) fn new(order_id: OrderId) -> Self {
        Self {
            order_id,
            match_result: None,
            cancel_reason: None,
        }
    }

    /// Return this order outcome with the match result set
    pub(crate) fn with_match_result(mut self, match_result: MatchResult) -> Self {
        self.match_result = Some(match_result);
        self
    }

    /// Return this order outcome with the reason the order was cancelled set
    pub(crate) fn with_cancel_reason(mut self, cancel_reason: CancelReason) -> Self {
        self.cancel_reason = Some(cancel_reason);
        self
    }

    /// Get the ID of the order
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }

    /// Get the match result if the order was matched
    pub fn match_result(&self) -> Option<&MatchResult> {
        self.match_result.as_ref()
    }

    /// Get the reason the order was cancelled, if it was cancelled
    pub fn cancel_reason(&self) -> Option<&CancelReason> {
        self.cancel_reason.as_ref()
    }
}

#[cfg(test)]
mod tests_order_outcome {
    use super::*;
    use crate::{
        Quantity, Side,
        report::{CancelReason, MatchResult},
    };

    fn create_order_outcome() -> OrderOutcome {
        OrderOutcome::new(OrderId(1))
    }

    #[test]
    fn test_order_id() {
        assert_eq!(create_order_outcome().order_id(), OrderId(1));
    }

    #[test]
    fn test_cancel_reason() {
        let mut order_outcome = create_order_outcome();
        assert_eq!(order_outcome.cancel_reason(), None);

        let cancel_reason = CancelReason::InsufficientLiquidity {
            available: Quantity(50),
        };
        order_outcome = order_outcome.with_cancel_reason(cancel_reason.clone());
        assert_eq!(order_outcome.cancel_reason(), Some(&cancel_reason));
    }

    #[test]
    fn test_match_result() {
        let mut order_outcome = create_order_outcome();
        assert!(order_outcome.match_result().is_none());

        let match_result = MatchResult::new(Side::Buy);
        order_outcome = order_outcome.with_match_result(match_result);
        assert!(order_outcome.match_result().is_some());
    }
}
