use crate::{book::OrderBook, commands::*, execution::*};

use serde::{Deserialize, Serialize};

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> OrderBook<E> {
    /// Execute an amend command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_amend(&mut self, cmd: &AmendCmd) -> Result<AmendReport, ExecutionError> {
        match &cmd.changes {
            AmendChanges::Limit(amend) => self.amend_limit_order(cmd.order_id, amend),
            AmendChanges::Pegged(amend) => self.amend_pegged_order(cmd.order_id, amend),
        }
    }

    /// Amend a limit order
    fn amend_limit_order(
        &mut self,
        _order_id: u64,
        _amend: &LimitAmend,
    ) -> Result<AmendReport, ExecutionError> {
        todo!()
    }

    /// Amend a pegged order
    fn amend_pegged_order(
        &mut self,
        _order_id: u64,
        _amend: &PeggedAmend,
    ) -> Result<AmendReport, ExecutionError> {
        todo!()
    }
}
