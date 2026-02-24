use crate::{book::OrderBook, commands::*, execution::*};

use serde::{Deserialize, Serialize};

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> OrderBook<E> {
    /// Execute a cancel command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_cancel(&mut self, cancel_cmd: CancelCmd) -> Result<(), ExecutionError> {
        match cancel_cmd.order_kind {
            OrderKind::Limit => self.cancel_limit_order(cancel_cmd.order_id),
            OrderKind::Pegged => self.cancel_pegged_order(cancel_cmd.order_id),
        }
    }

    /// Cancel a limit order
    fn cancel_limit_order(&mut self, order_id: u64) -> Result<(), ExecutionError> {
        _ = order_id;
        todo!()
    }

    /// Cancel a pegged order
    fn cancel_pegged_order(&mut self, order_id: u64) -> Result<(), ExecutionError> {
        _ = order_id;
        todo!()
    }
}
