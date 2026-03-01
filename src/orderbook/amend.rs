use crate::{command::*, orderbook::OrderBook, report::*};

impl OrderBook {
    /// Execute an amend command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_amend(&mut self, meta: CommandMeta, cmd: &AmendCmd) -> CommandOutcome {
        let result = match &cmd.patch {
            AmendPatch::Limit(patch) => self.amend_limit_order(meta, cmd.order_id, patch),
            AmendPatch::Pegged(patch) => self.amend_pegged_order(meta, cmd.order_id, patch),
        };

        match result {
            Ok(report) => CommandOutcome::Applied(CommandReport::Amend(report)),
            Err(error) => CommandOutcome::Rejected(error),
        }
    }

    /// Amend a limit order
    fn amend_limit_order(
        &mut self,
        _meta: CommandMeta,
        _order_id: u64,
        _patch: &LimitOrderPatch,
    ) -> Result<AmendReport, CommandError> {
        todo!()
    }

    /// Amend a pegged order
    fn amend_pegged_order(
        &mut self,
        _meta: CommandMeta,
        _order_id: u64,
        _patch: &PeggedOrderPatch,
    ) -> Result<AmendReport, CommandError> {
        todo!()
    }
}
