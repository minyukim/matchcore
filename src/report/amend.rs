use super::{OrderProcessingResult, OrderProcessingResults};

use serde::{Deserialize, Serialize};

/// Represents the report for the amendment of an existing order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendReport {
    /// The new order ID, if the amend command resulted in an order replacement
    /// caused by losing time-priority due to price change or quantity increase
    new_order_id: Option<u64>,
    /// The results of the order processing
    order_processing_results: OrderProcessingResults,
}

#[allow(unused)]
impl AmendReport {
    /// Create a new amend report
    pub(crate) fn new(new_order_id: Option<u64>, amended_order: OrderProcessingResult) -> Self {
        Self {
            new_order_id,
            order_processing_results: OrderProcessingResults::new(amended_order),
        }
    }

    /// Return this amend report with the triggered orders set
    pub(crate) fn with_triggered_orders(
        mut self,
        triggered_orders: Vec<OrderProcessingResult>,
    ) -> Self {
        self.order_processing_results
            .set_triggered_orders(triggered_orders);
        self
    }

    /// Get the new order ID, if the amend command resulted in an order replacement
    /// caused by losing time-priority due to price change or quantity increase
    pub fn new_order_id(&self) -> Option<u64> {
        self.new_order_id
    }

    /// Get the result for the order explicitly amended by the command
    pub fn amended_order(&self) -> &OrderProcessingResult {
        self.order_processing_results.primary_order()
    }

    /// Get the other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    pub fn triggered_orders(&self) -> &[OrderProcessingResult] {
        self.order_processing_results.triggered_orders()
    }
}
