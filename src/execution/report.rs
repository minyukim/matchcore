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
    pub order_processing_results: Vec<OrderProcessingResult>,
}

/// Represents a report to amend an existing order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendReport {
    /// The new order ID, if the amend command resulted in an order replacement
    /// caused by losing time-priority due to price change or quantity increase
    pub new_order_id: Option<u64>,
    /// The results of processing the orders triggered by the amend command
    pub order_processing_results: Vec<OrderProcessingResult>,
}
