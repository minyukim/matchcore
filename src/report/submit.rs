use crate::report::{OrderProcessingResult, OrderProcessingResults};

use serde::{Deserialize, Serialize};

/// Represents the report for the submission of a new order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReport {
    /// The results of the order processing
    order_processing_results: OrderProcessingResults,
}

impl SubmitReport {
    /// Create a new submit report
    pub(crate) fn new(submitted_order: OrderProcessingResult) -> Self {
        Self {
            order_processing_results: OrderProcessingResults::new(submitted_order),
        }
    }

    /// Return this submit report with the triggered orders set
    pub(crate) fn with_triggered_orders(
        mut self,
        triggered_orders: Vec<OrderProcessingResult>,
    ) -> Self {
        self.order_processing_results
            .set_triggered_orders(triggered_orders);
        self
    }

    /// Get the result for the order explicitly submitted by the command
    pub fn submitted_order(&self) -> &OrderProcessingResult {
        self.order_processing_results.primary_order()
    }

    /// Get the other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    pub fn triggered_orders(&self) -> &[OrderProcessingResult] {
        self.order_processing_results.triggered_orders()
    }
}
