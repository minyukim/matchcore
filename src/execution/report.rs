use crate::execution::OrderProcessingResult;

use serde::{Deserialize, Serialize};

/// Represents a report of the execution of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionReport {
    /// A report to submit a new order
    Submit(SubmitReport),
    /// A report to amend an existing order
    Amend(AmendReport),
    /// A report to cancel an existing order
    Cancel,
}

/// Represents a report to submit a new order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReport {
    /// The results of processing the orders triggered by the submit command
    order_processing_results: Vec<OrderProcessingResult>,
}

impl SubmitReport {
    /// Create a new submit report
    pub fn new(order_processing_results: Vec<OrderProcessingResult>) -> Self {
        Self {
            order_processing_results,
        }
    }

    /// Get the results of processing the orders triggered by the submit command
    pub fn order_processing_results(&self) -> &[OrderProcessingResult] {
        &self.order_processing_results
    }
}

/// Represents a report to amend an existing order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendReport {
    /// The new order ID, if the amend command resulted in an order replacement
    /// caused by losing time-priority due to price change or quantity increase
    new_order_id: Option<u64>,
    /// The results of processing the orders triggered by the amend command
    order_processing_results: Vec<OrderProcessingResult>,
}

impl AmendReport {
    /// Create a new amend report
    pub fn new(
        new_order_id: Option<u64>,
        order_processing_results: Vec<OrderProcessingResult>,
    ) -> Self {
        Self {
            new_order_id,
            order_processing_results,
        }
    }

    /// Get the new order ID, if the amend command resulted in an order replacement
    /// caused by losing time-priority due to price change or quantity increase
    pub fn new_order_id(&self) -> Option<u64> {
        self.new_order_id
    }

    /// Get the results of processing the orders triggered by the amend command
    pub fn order_processing_results(&self) -> &[OrderProcessingResult] {
        &self.order_processing_results
    }
}
