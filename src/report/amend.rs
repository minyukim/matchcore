use crate::report::OrderProcessingResult;

use serde::{Deserialize, Serialize};

/// Represents a report to amend an existing order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendReport {
    /// The new order ID, if the amend command resulted in an order replacement
    /// caused by losing time-priority due to price change or quantity increase
    new_order_id: Option<u64>,
    /// Result for the order explicitly amended by the command
    amended_order: OrderProcessingResult,
    /// Other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    triggered_orders: Vec<OrderProcessingResult>,
}

impl AmendReport {
    /// Create a new amend report
    pub fn new(
        new_order_id: Option<u64>,
        amended_order: OrderProcessingResult,
        triggered_orders: Vec<OrderProcessingResult>,
    ) -> Self {
        Self {
            new_order_id,
            amended_order,
            triggered_orders,
        }
    }

    /// Get the new order ID, if the amend command resulted in an order replacement
    /// caused by losing time-priority due to price change or quantity increase
    pub fn new_order_id(&self) -> Option<u64> {
        self.new_order_id
    }

    /// Get the result for the order explicitly amended by the command
    pub fn amended_order(&self) -> &OrderProcessingResult {
        &self.amended_order
    }

    /// Get the other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    pub fn triggered_orders(&self) -> &[OrderProcessingResult] {
        &self.triggered_orders
    }
}
