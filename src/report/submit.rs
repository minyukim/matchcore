use crate::report::OrderProcessingResult;

use serde::{Deserialize, Serialize};

/// Represents the report for the submission of a new order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReport {
    /// Result for the order explicitly submitted by the command
    submitted_order: OrderProcessingResult,
    /// Other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    triggered_orders: Vec<OrderProcessingResult>,
}

impl SubmitReport {
    /// Create a new submit report
    pub fn new(
        submitted_order: OrderProcessingResult,
        triggered_orders: Vec<OrderProcessingResult>,
    ) -> Self {
        Self {
            submitted_order,
            triggered_orders,
        }
    }

    /// Get the result for the order explicitly submitted by the command
    pub fn submitted_order(&self) -> &OrderProcessingResult {
        &self.submitted_order
    }

    /// Get the other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    pub fn triggered_orders(&self) -> &[OrderProcessingResult] {
        &self.triggered_orders
    }
}
