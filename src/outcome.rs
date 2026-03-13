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

use serde::{Deserialize, Serialize};

/// Represents the outcome of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        )));
        println!("{}", outcome);
        assert_eq!(
            outcome.to_string(),
            "order submitted: effects:\n  target order(1):\n    not matched\n    not cancelled\n"
        );

        let outcome = CommandOutcome::Applied(CommandReport::Amend(CommandEffects::new(
            OrderOutcome::new(OrderId(1)),
        )));
        println!("{}", outcome);
        assert_eq!(
            outcome.to_string(),
            "order amended: effects:\n  target order(1):\n    not matched\n    not cancelled\n"
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

        let outcome = CommandOutcome::Applied(CommandReport::Submit(
            CommandEffects::new(
                OrderOutcome::new(OrderId(1)).with_match_result(MatchResult::new(Side::Buy)),
            )
            .with_triggered_orders(vec![
                OrderOutcome::new(OrderId(2)),
                OrderOutcome::new(OrderId(3)),
            ])
            .with_triggered_orders(vec![
                OrderOutcome::new(OrderId(2)).with_match_result(MatchResult::new(Side::Buy)),
                OrderOutcome::new(OrderId(3)).with_cancel_reason(CancelReason::PostOnlyWouldTake),
            ]),
        ));
        println!("{}", outcome);
        assert_eq!(
            outcome.to_string(),
            "order submitted: effects:\n  target order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(2):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(3):\n    not matched\n    cancelled: post-only order would remove liquidity\n"
        );
    }
}
