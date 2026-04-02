use super::OrderOutcome;
use crate::utils::write_indented;

use std::fmt;

/// Effects from the execution of a command
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEffects {
    /// Outcome of the order that was explicitly targeted by the command
    target_order: OrderOutcome,

    /// Outcomes of the other orders whose state changed as a consequence
    /// (e.g., inactive pegged orders becoming active)
    triggered_orders: Vec<OrderOutcome>,
}

impl CommandEffects {
    /// Create a new command effects
    pub(crate) fn new(target_order: OrderOutcome, triggered_orders: Vec<OrderOutcome>) -> Self {
        Self {
            target_order,
            triggered_orders,
        }
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
mod tests {
    use super::*;
    use crate::{CancelReason, MatchResult, OrderId, OrderOutcome, Side};

    #[test]
    fn test_display() {
        let command_effects = CommandEffects::new(OrderOutcome::new(OrderId(1)), Vec::new());
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  target order(1):\n    not matched\n    not cancelled\n"
        );

        let command_effects = CommandEffects::new(
            OrderOutcome::new(OrderId(1)),
            vec![OrderOutcome::new(OrderId(2))],
        );
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  target order(1):\n    not matched\n    not cancelled\n  triggered order(2):\n    not matched\n    not cancelled\n"
        );

        let command_effects = CommandEffects::new(
            OrderOutcome::new(OrderId(1)),
            vec![OrderOutcome::new(OrderId(2)), OrderOutcome::new(OrderId(3))],
        );
        println!("{}", command_effects);
        assert_eq!(
            command_effects.to_string(),
            "effects:\n  target order(1):\n    not matched\n    not cancelled\n  triggered order(2):\n    not matched\n    not cancelled\n  triggered order(3):\n    not matched\n    not cancelled\n"
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
            "effects:\n  target order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(2):\n    not matched\n    not cancelled\n  triggered order(3):\n    not matched\n    not cancelled\n"
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
            "effects:\n  target order(1):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(2):\n    matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n    not cancelled\n  triggered order(3):\n    not matched\n    cancelled: post-only order would remove liquidity\n"
        );
    }
}
