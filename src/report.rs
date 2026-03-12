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

use crate::{CommandMeta, utils::write_indented};

use std::fmt;

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

impl fmt::Display for CommandExecutionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "report:")?;
        writeln!(f, "  sequence number: {}", self.meta().sequence_number)?;
        writeln!(f, "  timestamp: {}", self.meta().timestamp)?;
        write_indented(f, &self.outcome().to_string(), 2)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests_command_execution_report {
    use super::*;
    use crate::{CommandMeta, CommandOutcome, OrderId, OrderOutcome, SequenceNumber, Timestamp};

    #[test]
    fn test_display() {
        let command_execution_report = CommandExecutionReport::new(
            CommandMeta {
                sequence_number: SequenceNumber(1),
                timestamp: Timestamp(1000),
            },
            CommandOutcome::Applied(AppliedCommand::Submit(CommandEffects::new(
                OrderOutcome::new(OrderId(1)),
            ))),
        );
        println!("{}", command_execution_report);
        assert_eq!(
            command_execution_report.to_string(),
            "report:\n  sequence number: 1\n  timestamp: 1000\n  submit applied, effects:\n    target order(1):\n      not matched\n      not cancelled\n"
        );
    }
}
