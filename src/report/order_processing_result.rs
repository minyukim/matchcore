use crate::{CancelReason, report::MatchResult};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct OrderProcessingResults {
    /// Result for the primary order explicitly stated in the command
    primary_order: OrderProcessingResult,
    /// Other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    triggered_orders: Vec<OrderProcessingResult>,
}

impl OrderProcessingResults {
    /// Create a new order processing results
    pub(super) fn new(primary_order: OrderProcessingResult) -> Self {
        Self {
            primary_order,
            triggered_orders: Vec::new(),
        }
    }

    pub(super) fn set_triggered_orders(&mut self, triggered_orders: Vec<OrderProcessingResult>) {
        self.triggered_orders = triggered_orders;
    }

    /// Get the result for the primary order explicitly stated in the command
    pub(super) fn primary_order(&self) -> &OrderProcessingResult {
        &self.primary_order
    }

    /// Get the other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    pub(super) fn triggered_orders(&self) -> &[OrderProcessingResult] {
        &self.triggered_orders
    }
}

/// Result of processing a taker order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderProcessingResult {
    /// The ID of the order
    order_id: u64,
    /// The match result if the order was matched
    match_result: Option<MatchResult>,
    /// The reason the order was cancelled, if it was cancelled
    cancel_reason: Option<CancelReason>,
}

impl OrderProcessingResult {
    /// Create a new order processing result
    pub fn new(order_id: u64) -> Self {
        Self {
            order_id,
            match_result: None,
            cancel_reason: None,
        }
    }

    /// Get the ID of the order
    pub fn order_id(&self) -> u64 {
        self.order_id
    }

    /// Get the match result if the order was matched
    pub fn match_result(&self) -> Option<&MatchResult> {
        self.match_result.as_ref()
    }

    /// Return this order processing result with the match result set
    #[allow(unused)]
    pub(crate) fn with_match_result(mut self, match_result: MatchResult) -> Self {
        self.match_result = Some(match_result);
        self
    }

    /// Get the reason the order was cancelled, if it was cancelled
    pub fn cancel_reason(&self) -> Option<&CancelReason> {
        self.cancel_reason.as_ref()
    }

    /// Return this order processing result with the reason the order was cancelled set
    #[allow(unused)]
    pub(crate) fn with_cancel_reason(mut self, cancel_reason: CancelReason) -> Self {
        self.cancel_reason = Some(cancel_reason);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Side,
        report::{CancelReason, MatchResult},
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
        order_processing_result = order_processing_result.with_cancel_reason(cancel_reason.clone());
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
        order_processing_result = order_processing_result.with_match_result(match_result);
        assert!(order_processing_result.match_result().is_some());
    }
}
