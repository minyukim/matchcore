use super::CommandEffects;

use std::fmt;

/// Report from the execution of a command
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandReport {
    /// The effects of the submission of a new order
    Submit(CommandEffects),
    /// The effects of the amendment of an existing order
    Amend(CommandEffects),
    /// The cancellation of an existing order
    Cancel,
}

impl fmt::Display for CommandReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandReport::Submit(effects) => write!(f, "order submitted: {}", effects)?,
            CommandReport::Amend(effects) => write!(f, "order amended: {}", effects)?,
            CommandReport::Cancel => writeln!(f, "order cancelled")?,
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
        let report = CommandReport::Submit(CommandEffects::new(OrderOutcome::new(OrderId(1))));
        println!("{}", report);
        assert_eq!(
            report.to_string(),
            "order submitted: effects:\n  target order(1):\n    not matched\n    not cancelled\n"
        );

        let report = CommandReport::Amend(CommandEffects::new(OrderOutcome::new(OrderId(1))));
        println!("{}", report);
        assert_eq!(
            report.to_string(),
            "order amended: effects:\n  target order(1):\n    not matched\n    not cancelled\n"
        );

        let report = CommandReport::Cancel;
        println!("{}", report);
        assert_eq!(report.to_string(), "order cancelled\n");

        let mut order_outcome1 = OrderOutcome::new(OrderId(1));
        order_outcome1.set_match_result(MatchResult::new(Side::Buy));
        let mut order_outcome2 = OrderOutcome::new(OrderId(2));
        order_outcome2.set_match_result(MatchResult::new(Side::Buy));
        let mut order_outcome3 = OrderOutcome::new(OrderId(3));
        order_outcome3.set_cancel_reason(CancelReason::PostOnlyWouldTake);

        let mut command_effects = CommandEffects::new(order_outcome1);
        command_effects.add_triggered_order(order_outcome2);
        command_effects.add_triggered_order(order_outcome3);
        let report = CommandReport::Submit(command_effects);
        println!("{}", report);
        assert_eq!(
            report.to_string(),
            "order submitted: effects:\n  target order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(2):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(3):\n    not matched\n    cancelled: post-only order would remove liquidity\n"
        );
    }
}
