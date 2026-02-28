mod amend;
mod cancel;
mod error;
mod submit;
mod validation;

pub use amend::*;
pub use cancel::*;
pub use error::*;
pub use submit::*;

use serde::{Deserialize, Serialize};

/// Represents a top-level command for all command and order kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command {
    /// The common metadata for all command kinds
    pub meta: CommandMeta,
    /// The kind of command
    pub kind: CommandKind,
}

/// Represents the common metadata for all command kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandMeta {
    /// The sequence number of the command
    pub sequence_number: u64,
    /// The timestamp of the command
    pub timestamp: u64,
}

/// Represents the kind of command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandKind {
    /// A command to submit a new order
    Submit(SubmitCmd),
    /// A command to amend an existing order
    Amend(AmendCmd),
    /// A command to cancel an existing order
    Cancel(CancelCmd),
}
