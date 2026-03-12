use super::OrderOutcome;
use crate::RejectReason;

use serde::{Deserialize, Serialize};

/// Represents the outcome of the execution of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandOutcome {
    /// The command was applied successfully
    Applied(AppliedCommand),
    /// The command was rejected due to an error
    Rejected(RejectReason),
}

/// Represents the kinds of the applied command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppliedCommand {
    /// The effects of the submission of a new order
    Submit(CommandEffects),
    /// The effects of the amendment of an existing order
    Amend(CommandEffects),
    /// The cancellation of an existing order
    Cancel,
}

/// Represents the effects of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEffects {
    /// Outcome of the order that was explicitly targeted by the command
    /// Note that for the amend command, the order ID would be different from the original ID
    /// if the order was replaced due to losing time-priority (price change or quantity increase)
    target_order: OrderOutcome,

    /// Outcomes of the other orders whose state changed as a consequence
    /// (e.g., inactive pegged orders becoming active)
    triggered_orders: Vec<OrderOutcome>,
}

impl CommandEffects {
    /// Create a new command effects
    pub(crate) fn new(target_order: OrderOutcome) -> Self {
        Self {
            target_order,
            triggered_orders: Vec::new(),
        }
    }

    pub(crate) fn with_triggered_orders(mut self, triggered_orders: Vec<OrderOutcome>) -> Self {
        self.triggered_orders = triggered_orders;
        self
    }

    /// Get the outcome of the order that was explicitly targeted by the command
    pub fn target_order(&self) -> &OrderOutcome {
        &self.target_order
    }

    /// Get the outcomes of the other orders whose state changed as a consequence
    /// (e.g., inactive pegged orders becoming active)
    pub fn triggered_orders(&self) -> &[OrderOutcome] {
        &self.triggered_orders
    }
}
