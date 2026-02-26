use crate::{
    CommandError,
    report::{AmendReport, SubmitReport},
};

use serde::{Deserialize, Serialize};

/// Represents the outcome of the execution of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandOutcome {
    /// The command was applied successfully
    Applied(CommandReport),
    /// The command was rejected due to an error
    Rejected(CommandError),
}

/// Represents the report of the execution of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandReport {
    /// The report for the submission of a new order
    Submit(SubmitReport),
    /// The report for the amendment of an existing order
    Amend(AmendReport),
    /// The report for the cancellation of an existing order
    Cancel,
}
