use super::{OrderProcessingResult, OrderProcessingResults, SubmitReport};
use crate::OrderId;

use serde::{Deserialize, Serialize};

/// Represents the report for the amendment of an existing order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendReport {
    /// The new order ID, if the amend command resulted in an order replacement
    /// caused by losing time-priority due to price change or quantity increase
    new_order_id: Option<OrderId>,
    /// The results of the order processing
    order_processing_results: OrderProcessingResults,
}

#[allow(unused)]
impl AmendReport {
    /// Create a new amend report
    pub(crate) fn new(amended_order: OrderProcessingResult) -> Self {
        Self {
            new_order_id: None,
            order_processing_results: OrderProcessingResults::new(amended_order),
        }
    }

    /// Return this amend report with the new order ID set
    pub(crate) fn with_new_order_id(mut self, new_order_id: OrderId) -> Self {
        self.new_order_id = Some(new_order_id);
        self
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
    pub fn new_order_id(&self) -> Option<OrderId> {
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

impl From<OrderProcessingResults> for AmendReport {
    fn from(results: OrderProcessingResults) -> Self {
        Self {
            new_order_id: None,
            order_processing_results: results,
        }
    }
}

impl From<SubmitReport> for AmendReport {
    fn from(report: SubmitReport) -> Self {
        Self {
            new_order_id: None,
            order_processing_results: report.into(),
        }
    }
}
