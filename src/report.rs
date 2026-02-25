mod amend;
mod cancel_reason;
mod error;
mod match_result;
mod order_processing_result;
mod submit;
mod trade;

pub use amend::*;
pub use cancel_reason::*;
pub use error::*;
pub use match_result::*;
pub use order_processing_result::*;
pub use submit::*;
pub use trade::*;

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
