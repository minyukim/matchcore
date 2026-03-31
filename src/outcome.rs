//! Outcome specifications for the command execution
//!
//! The `CommandOutcome` type is the top-level outcome of the execution of all kinds of commands,
//! and is the only output produced by the order book.

mod cancel_reason;
mod command_effects;
mod command_failure;
mod command_report;
mod match_result;
mod order_outcome;
mod trade;

pub use cancel_reason::*;
pub use command_effects::*;
pub use command_failure::*;
pub use command_report::*;
pub use match_result::*;
pub use order_outcome::*;
pub use trade::*;

use std::fmt;

/// Represents the outcome of a command execution
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    Applied(CommandReport),
    Rejected(CommandFailure),
}

impl fmt::Display for CommandOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandOutcome::Applied(report) => write!(f, "{}", report)?,
            CommandOutcome::Rejected(failure) => writeln!(f, "command rejected: {}", failure)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CancelReason, CommandEffects, CommandError, MatchResult, OrderId, OrderOutcome, Side,
    };

    #[test]
    fn test_display() {
        let outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId(1)),
            Vec::new(),
        )));
        println!("{}", outcome);
        assert_eq!(
            outcome.to_string(),
            "order submitted: effects:\n  primary order(1):\n    not matched\n    not cancelled\n"
        );

        let outcome = CommandOutcome::Applied(CommandReport::Amend(CommandEffects::new(
            OrderOutcome::new(OrderId(1)),
            Vec::new(),
        )));
        println!("{}", outcome);
        assert_eq!(
            outcome.to_string(),
            "order amended: effects:\n  primary order(1):\n    not matched\n    not cancelled\n"
        );

        let outcome = CommandOutcome::Applied(CommandReport::Cancel);
        println!("{}", outcome);
        assert_eq!(outcome.to_string(), "order cancelled\n");

        let outcome =
            CommandOutcome::Rejected(CommandFailure::InvalidCommand(CommandError::ZeroPrice));
        println!("{}", outcome);
        assert_eq!(
            outcome.to_string(),
            "command rejected: invalid command: price is zero\n"
        );

        let mut order_outcome1 = OrderOutcome::new(OrderId(1));
        order_outcome1.set_match_result(MatchResult::new(Side::Buy));
        let mut order_outcome2 = OrderOutcome::new(OrderId(2));
        order_outcome2.set_match_result(MatchResult::new(Side::Buy));
        let mut order_outcome3 = OrderOutcome::new(OrderId(3));
        order_outcome3.set_cancel_reason(CancelReason::PostOnlyWouldTake);

        let command_effects =
            CommandEffects::new(order_outcome1, vec![order_outcome2, order_outcome3]);
        let outcome = CommandOutcome::Applied(CommandReport::Submit(command_effects));
        println!("{}", outcome);
        assert_eq!(
            outcome.to_string(),
            "order submitted: effects:\n  primary order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  cascading order(2):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  cascading order(3):\n    not matched\n    cancelled: post-only order would remove liquidity\n"
        );
    }
}
