use super::OrderOutcome;
use crate::{RejectReason, utils::write_indented};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents the outcome of the execution of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandOutcome {
    /// The command was applied successfully
    Applied(AppliedCommand),
    /// The command was rejected due to an error
    Rejected(RejectReason),
}

impl fmt::Display for CommandOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandOutcome::Applied(applied_command) => match applied_command {
                AppliedCommand::Submit(command_effects) => {
                    write!(f, "submit applied, {}", command_effects)?;
                }
                AppliedCommand::Amend(command_effects) => {
                    write!(f, "amend applied, {}", command_effects)?;
                }
                AppliedCommand::Cancel => writeln!(f, "cancel applied")?,
            },
            CommandOutcome::Rejected(reject_reason) => writeln!(f, "rejected: {}", reject_reason)?,
        }
        Ok(())
    }
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

impl fmt::Display for CommandEffects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "effects:")?;

        write_indented(f, &format!("target {}", self.target_order()), 2)?;

        for triggered_order in self.triggered_orders() {
            write_indented(f, &format!("triggered {}", triggered_order), 2)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_command_outcome {
    use super::*;
    use crate::{
        CancelReason, CommandEffects, CommandError, MatchResult, OrderId, OrderOutcome, Side,
    };

    #[test]
    fn test_display() {
        let command_outcome = CommandOutcome::Applied(AppliedCommand::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId(1)),
        )));
        println!("{}", command_outcome);
        assert_eq!(
            command_outcome.to_string(),
            "submit applied, effects:\n  target order(1):\n    not matched\n    not cancelled\n"
        );

        let command_outcome = CommandOutcome::Applied(AppliedCommand::Amend(CommandEffects::new(
            OrderOutcome::new(OrderId(1)),
        )));
        println!("{}", command_outcome);
        assert_eq!(
            command_outcome.to_string(),
            "amend applied, effects:\n  target order(1):\n    not matched\n    not cancelled\n"
        );

        let command_outcome = CommandOutcome::Applied(AppliedCommand::Cancel);
        println!("{}", command_outcome);
        assert_eq!(command_outcome.to_string(), "cancel applied\n");

        let command_outcome =
            CommandOutcome::Rejected(RejectReason::InvalidCommand(CommandError::ZeroPrice));
        println!("{}", command_outcome);
        assert_eq!(
            command_outcome.to_string(),
            "rejected: invalid command: price is zero\n"
        );

        let command_outcome = CommandOutcome::Applied(AppliedCommand::Submit(
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
        println!("{}", command_outcome);
        assert_eq!(
            command_outcome.to_string(),
            "submit applied, effects:\n  target order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(2):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(3):\n    not matched\n    cancelled: post-only order would remove liquidity\n"
        );
    }
}

#[cfg(test)]
mod tests_command_effects {
    use super::*;
    use crate::{CancelReason, MatchResult, OrderId, OrderOutcome, Side};

    #[test]
    fn test_display() {
        let command_effects = CommandEffects::new(OrderOutcome::new(OrderId(1)));
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  target order(1):\n    not matched\n    not cancelled\n"
        );

        let command_effects =
            command_effects.with_triggered_orders(vec![OrderOutcome::new(OrderId(2))]);
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  target order(1):\n    not matched\n    not cancelled\n  triggered order(2):\n    not matched\n    not cancelled\n"
        );

        let command_effects = command_effects.with_triggered_orders(vec![
            OrderOutcome::new(OrderId(2)),
            OrderOutcome::new(OrderId(3)),
        ]);
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  target order(1):\n    not matched\n    not cancelled\n  triggered order(2):\n    not matched\n    not cancelled\n  triggered order(3):\n    not matched\n    not cancelled\n"
        );

        let command_effects = CommandEffects::new(
            OrderOutcome::new(OrderId(1)).with_match_result(MatchResult::new(Side::Buy)),
        )
        .with_triggered_orders(vec![
            OrderOutcome::new(OrderId(2)),
            OrderOutcome::new(OrderId(3)),
        ]);
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  target order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(2):\n    not matched\n    not cancelled\n  triggered order(3):\n    not matched\n    not cancelled\n"
        );

        let command_effects = command_effects.with_triggered_orders(vec![
            OrderOutcome::new(OrderId(2)).with_match_result(MatchResult::new(Side::Buy)),
            OrderOutcome::new(OrderId(3)).with_cancel_reason(CancelReason::PostOnlyWouldTake),
        ]);
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  target order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(2):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(3):\n    not matched\n    cancelled: post-only order would remove liquidity\n"
        );
    }
}
