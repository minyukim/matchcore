use super::{CancelReason, MatchResult};
use crate::{OrderId, utils::write_indented};

use std::fmt;

use serde::{Deserialize, Serialize};

/// Outcome of the order execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderOutcome {
    /// The ID of the order
    order_id: OrderId,
    /// The match result if the order was matched
    match_result: Option<MatchResult>,
    /// The reason the order was cancelled, if it was cancelled
    cancel_reason: Option<CancelReason>,
}

impl OrderOutcome {
    /// Create a new order outcome
    pub(crate) fn new(order_id: OrderId) -> Self {
        Self {
            order_id,
            match_result: None,
            cancel_reason: None,
        }
    }

    /// Return this order outcome with the match result set
    pub(crate) fn with_match_result(mut self, match_result: MatchResult) -> Self {
        self.match_result = Some(match_result);
        self
    }

    /// Return this order outcome with the reason the order was cancelled set
    pub(crate) fn with_cancel_reason(mut self, cancel_reason: CancelReason) -> Self {
        self.cancel_reason = Some(cancel_reason);
        self
    }

    /// Get the ID of the order
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }

    /// Get the match result if the order was matched
    pub fn match_result(&self) -> Option<&MatchResult> {
        self.match_result.as_ref()
    }

    /// Get the reason the order was cancelled, if it was cancelled
    pub fn cancel_reason(&self) -> Option<&CancelReason> {
        self.cancel_reason.as_ref()
    }
}

impl fmt::Display for OrderOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "order({}):", self.order_id())?;

        match self.match_result() {
            Some(match_result) => {
                write_indented(f, &format!("matched: {}", match_result), 2)?;
            }
            None => {
                writeln!(f, "  not matched")?;
            }
        }

        match self.cancel_reason() {
            Some(cancel_reason) => {
                writeln!(f, "  cancelled: {}", cancel_reason)?;
            }
            None => {
                writeln!(f, "  not cancelled")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_order_outcome {
    use super::*;
    use crate::{Price, Quantity, Side, Trade};

    fn create_order_outcome() -> OrderOutcome {
        OrderOutcome::new(OrderId(1))
    }

    #[test]
    fn test_order_id() {
        assert_eq!(create_order_outcome().order_id(), OrderId(1));
    }

    #[test]
    fn test_cancel_reason() {
        let mut order_outcome = create_order_outcome();
        assert_eq!(order_outcome.cancel_reason(), None);

        let cancel_reason = CancelReason::InsufficientLiquidity {
            available: Quantity(50),
        };
        order_outcome = order_outcome.with_cancel_reason(cancel_reason.clone());
        assert_eq!(order_outcome.cancel_reason(), Some(&cancel_reason));
    }

    #[test]
    fn test_match_result() {
        let mut order_outcome = create_order_outcome();
        assert!(order_outcome.match_result().is_none());

        let match_result = MatchResult::new(Side::Buy);
        order_outcome = order_outcome.with_match_result(match_result);
        assert!(order_outcome.match_result().is_some());
    }

    #[test]
    fn test_display() {
        let order_outcome = create_order_outcome();
        println!("{}", order_outcome);
        assert_eq!(
            order_outcome.to_string(),
            "order(1):\n  not matched\n  not cancelled\n"
        );

        let order_outcome = create_order_outcome().with_match_result(MatchResult::new(Side::Buy));
        println!("{}", order_outcome);
        assert_eq!(
            order_outcome.to_string(),
            "order(1):\n  matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n  not cancelled\n"
        );

        let mut match_result = MatchResult::new(Side::Buy);
        match_result.add_trade(Trade::new(OrderId(2), Price(99), Quantity(20)));
        match_result.add_trade(Trade::new(OrderId(3), Price(100), Quantity(30)));
        let order_outcome = create_order_outcome().with_match_result(match_result);
        println!("{}", order_outcome);
        assert_eq!(
            order_outcome.to_string(),
            "order(1):\n  matched: taker_side=BUY executed_quantity=50 executed_value=4980 trades=2\n    maker(2): 20@99\n    maker(3): 30@100\n  not cancelled\n"
        );

        let order_outcome =
            create_order_outcome().with_cancel_reason(CancelReason::InsufficientLiquidity {
                available: Quantity(50),
            });
        println!("{}", order_outcome);
        assert_eq!(
            order_outcome.to_string(),
            "order(1):\n  not matched\n  cancelled: insufficient liquidity: available=50\n"
        );

        let order_outcome = create_order_outcome()
            .with_match_result(MatchResult::new(Side::Buy))
            .with_cancel_reason(CancelReason::InsufficientLiquidity {
                available: Quantity(50),
            });
        println!("{}", order_outcome);
        assert_eq!(
            order_outcome.to_string(),
            "order(1):\n  matched: taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n  cancelled: insufficient liquidity: available=50\n"
        );
    }
}
