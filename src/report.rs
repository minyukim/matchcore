mod command;
mod match_result;
mod order_outcome;
mod reasons;
mod trade;

pub use command::*;
pub use match_result::*;
pub use order_outcome::*;
pub use reasons::*;
pub use trade::*;

use crate::CommandMeta;

use serde::{Deserialize, Serialize};

/// Represents the report of the execution of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionReport {
    /// The command metadata
    meta: CommandMeta,
    /// The outcome of the execution of the command
    outcome: CommandOutcome,
}

impl CommandExecutionReport {
    /// Create a new command execution report
    pub(crate) fn new(meta: CommandMeta, outcome: CommandOutcome) -> Self {
        Self { meta, outcome }
    }

    /// Get the command metadata
    pub fn meta(&self) -> CommandMeta {
        self.meta
    }

    /// Get the command outcome
    pub fn outcome(&self) -> &CommandOutcome {
        &self.outcome
    }
}
