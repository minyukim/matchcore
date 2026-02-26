use crate::{command::*, orderbook::OrderBook, report::*};

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
            Err(error) => CommandOutcome::Rejected(error),
        }
    }

    /// Cancel a limit order
    fn cancel_limit_order(&mut self, _order_id: u64) -> Result<(), CommandError> {
        todo!()
    }

    /// Cancel a pegged order
    fn cancel_pegged_order(&mut self, _order_id: u64) -> Result<(), CommandError> {
        todo!()
    }
}
