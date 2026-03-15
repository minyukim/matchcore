use super::OrderOutcome;
use crate::utils::write_indented;

use std::fmt;

use serde::{Deserialize, Serialize};

/// Effects from the execution of a command
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

    /// Return this command effects with the triggered orders set
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
mod tests {
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
