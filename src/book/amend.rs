use crate::{book::OrderBook, commands::*, execution::*};

use serde::{Deserialize, Serialize};

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> OrderBook<E> {
    /// Execute an amend command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_amend(
        &mut self,
        meta: CommandMeta,
        cmd: &AmendCmd,
    ) -> Result<AmendReport, ExecutionError> {
        match &cmd.patch {
            AmendPatch::Limit(patch) => self.amend_limit_order(meta, cmd.order_id, patch),
            AmendPatch::Pegged(patch) => self.amend_pegged_order(meta, cmd.order_id, patch),
        }
    }

    /// Amend a limit order
    fn amend_limit_order(
        &mut self,
        _meta: CommandMeta,
        _order_id: u64,
        _patch: &LimitPatch,
    ) -> Result<AmendReport, ExecutionError> {
        todo!()
    }

    /// Amend a pegged order
    fn amend_pegged_order(
        &mut self,
        _meta: CommandMeta,
        _order_id: u64,
        _patch: &PeggedPatch,
    ) -> Result<AmendReport, ExecutionError> {
        todo!()
    }
}
