use crate::{CancelReason, execution::MatchResult};

use serde::{Deserialize, Serialize};

/// Result of processing a taker order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderProcessingResult {
    /// The ID of the order
    order_id: u64,
    /// The reason the order was cancelled, if it was cancelled
    cancel_reason: Option<CancelReason>,
    /// The match result if the order was matched
    match_result: Option<MatchResult>,
}

impl OrderProcessingResult {
    /// Create a new order processing result
    pub fn new(order_id: u64) -> Self {
        Self {
            order_id,
            cancel_reason: None,
            match_result: None,
        }
    }

    /// Get the ID of the order
    pub fn order_id(&self) -> u64 {
        self.order_id
    }

    /// Get the reason the order was cancelled, if it was cancelled
    pub fn cancel_reason(&self) -> Option<&CancelReason> {
        self.cancel_reason.as_ref()
    }

    /// Set the reason the order was cancelled
    #[allow(unused)]
    pub(crate) fn set_cancel_reason(&mut self, cancel_reason: CancelReason) {
        self.cancel_reason = Some(cancel_reason);
    }

    /// Get the match result if the order was matched
    pub fn match_result(&self) -> Option<&MatchResult> {
        self.match_result.as_ref()
    }

    /// Set the match result if the order was matched
    #[allow(unused)]
    pub(crate) fn set_match_result(&mut self, match_result: MatchResult) {
        self.match_result = Some(match_result);
    }
}
