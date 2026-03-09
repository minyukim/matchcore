use super::OrderBook;
use crate::{OrderId, OrderKind, command::*, report::*};

impl OrderBook {
    /// Execute a cancel command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_cancel(&mut self, cmd: &CancelCmd) -> CommandOutcome {
        let result = match &cmd.order_kind {
            OrderKind::Limit => self.cancel_limit_order(cmd.order_id),
            OrderKind::Pegged => self.cancel_pegged_order(cmd.order_id),
        };

        match result {
            Ok(_) => CommandOutcome::Applied(CommandReport::Cancel),
            Err(reason) => CommandOutcome::Rejected(reason),
        }
    }

    /// Cancel a limit order
    fn cancel_limit_order(&mut self, id: OrderId) -> Result<(), RejectReason> {
        self.remove_limit_order(id)
            .ok_or(RejectReason::OrderNotFound)?;

        Ok(())
    }

    /// Cancel a pegged order
    fn cancel_pegged_order(&mut self, id: OrderId) -> Result<(), RejectReason> {
        self.remove_pegged_order(id)
            .ok_or(RejectReason::OrderNotFound)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CancelCmd, CommandOutcome, CommandReport, LimitOrder, OrderFlags, OrderId, OrderKind,
        PegReference, PeggedOrder, Price, Quantity, QuantityPolicy, RejectReason, Side,
        TimeInForce,
    };

    fn cancel(book: &mut OrderBook, order_id: OrderId, order_kind: OrderKind) -> CommandOutcome {
        book.execute_cancel(&CancelCmd {
            order_id,
            order_kind,
        })
    }

    #[test]
    fn cancel_limit_order_success() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let outcome = cancel(&mut book, OrderId(0), OrderKind::Limit);

        match outcome {
            CommandOutcome::Applied(CommandReport::Cancel) => {}
            other => panic!("expected applied cancel, got: {other:?}"),
        }
        assert!(!book.limit.orders.contains_key(&OrderId(0)));
        assert!(!book.limit.bid_levels.contains_key(&Price(100)));
    }

    #[test]
    fn cancel_limit_order_not_found() {
        let mut book = OrderBook::new("TEST");

        let outcome = cancel(&mut book, OrderId(999), OrderKind::Limit);

        match outcome {
            CommandOutcome::Rejected(RejectReason::OrderNotFound) => {}
            other => panic!("expected order not found, got: {other:?}"),
        }
    }

    #[test]
    fn cancel_pegged_order_success() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let outcome = cancel(&mut book, OrderId(0), OrderKind::Pegged);

        match outcome {
            CommandOutcome::Applied(CommandReport::Cancel) => {}
            other => panic!("expected applied cancel, got: {other:?}"),
        }
        assert!(!book.pegged.orders.contains_key(&OrderId(0)));
    }

    #[test]
    fn cancel_pegged_order_not_found() {
        let mut book = OrderBook::new("TEST");

        let outcome = cancel(&mut book, OrderId(999), OrderKind::Pegged);

        match outcome {
            CommandOutcome::Rejected(RejectReason::OrderNotFound) => {}
            other => panic!("expected order not found, got: {other:?}"),
        }
    }

    #[test]
    fn cancel_limit_order_with_wrong_kind_returns_not_found() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        // Requesting Pegged for a Limit order looks in pegged book, finds nothing
        let outcome = cancel(&mut book, OrderId(0), OrderKind::Pegged);

        match outcome {
            CommandOutcome::Rejected(RejectReason::OrderNotFound) => {}
            other => panic!("expected order not found, got: {other:?}"),
        }
        // Limit order should still exist
        assert!(book.limit.orders.contains_key(&OrderId(0)));
    }
}
