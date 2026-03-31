use super::OrderOutcome;
use crate::utils::write_indented;

use std::fmt;

/// Effects from the execution of a command
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEffects {
    /// Outcome of the explicitly targeted order in the command
    primary_outcome: OrderOutcome,

    /// Outcomes produced by cascading effects of the command
    /// (e.g., price-conditional orders and pegged orders becoming active)
    cascading_outcomes: Vec<OrderOutcome>,
}

impl CommandEffects {
    /// Create a new command effects
    pub(crate) fn new(
        primary_outcome: OrderOutcome,
        cascading_outcomes: Vec<OrderOutcome>,
    ) -> Self {
        Self {
            primary_outcome,
            cascading_outcomes,
        }
    }

    /// Get the outcome of the explicitly targeted order in the command
    pub fn primary_outcome(&self) -> &OrderOutcome {
        &self.primary_outcome
    }

    /// Get the outcomes produced by cascading effects of the command
    pub fn cascading_outcomes(&self) -> &[OrderOutcome] {
        &self.cascading_outcomes
    }
}

impl fmt::Display for CommandEffects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "effects:")?;

        write_indented(f, &format!("primary {}", self.primary_outcome()), 2)?;

        for cascading_outcome in self.cascading_outcomes() {
            write_indented(f, &format!("cascading {}", cascading_outcome), 2)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CancelReason, MatchResult, OrderId, OrderOutcome, Side};

    #[test]
    fn test_display() {
        let command_effects = CommandEffects::new(OrderOutcome::new(OrderId(1)), Vec::new());
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  primary order(1):\n    not matched\n    not cancelled\n"
        );

        let command_effects = CommandEffects::new(
            OrderOutcome::new(OrderId(1)),
            vec![OrderOutcome::new(OrderId(2))],
        );
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  primary order(1):\n    not matched\n    not cancelled\n  cascading order(2):\n    not matched\n    not cancelled\n"
        );

        let command_effects = CommandEffects::new(
            OrderOutcome::new(OrderId(1)),
            vec![OrderOutcome::new(OrderId(2)), OrderOutcome::new(OrderId(3))],
        );
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  primary order(1):\n    not matched\n    not cancelled\n  cascading order(2):\n    not matched\n    not cancelled\n  cascading order(3):\n    not matched\n    not cancelled\n"
        );

        let mut order_outcome = OrderOutcome::new(OrderId(1));
        order_outcome.set_match_result(MatchResult::new(Side::Buy));

        let command_effects = CommandEffects::new(
            order_outcome,
            vec![OrderOutcome::new(OrderId(2)), OrderOutcome::new(OrderId(3))],
        );
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  primary order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  cascading order(2):\n    not matched\n    not cancelled\n  cascading order(3):\n    not matched\n    not cancelled\n"
        );

        let mut order_outcome = OrderOutcome::new(OrderId(1));
        order_outcome.set_match_result(MatchResult::new(Side::Buy));

        let mut order_outcome2 = OrderOutcome::new(OrderId(2));
        order_outcome2.set_match_result(MatchResult::new(Side::Buy));
        let mut order_outcome3 = OrderOutcome::new(OrderId(3));
        order_outcome3.set_cancel_reason(CancelReason::PostOnlyWouldTake);

        let command_effects =
            CommandEffects::new(order_outcome, vec![order_outcome2, order_outcome3]);
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  primary order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  cascading order(2):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  cascading order(3):\n    not matched\n    cancelled: post-only order would remove liquidity\n"
        );
    }
}
