use crate::{command::*, orderbook::OrderBook, report::*};

impl OrderBook {
    /// Execute a submit command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_submit(&mut self, meta: CommandMeta, cmd: &SubmitCmd) -> CommandOutcome {
        let result = match &cmd.order {
            NewOrder::Market(order) => self.submit_market_order(meta, order),
            NewOrder::Limit(order) => self.submit_limit_order(meta, order),
            NewOrder::Pegged(order) => self.submit_pegged_order(meta, order),
        };

        match result {
            Ok(report) => CommandOutcome::Applied(CommandReport::Submit(report)),
            Err(error) => CommandOutcome::Rejected(error),
        }
    }

    /// Submit a market order
    fn submit_market_order(
        &mut self,
        _meta: CommandMeta,
        _order: &NewMarketOrder,
    ) -> Result<SubmitReport, CommandError> {
        todo!()
    }

    /// Submit a limit order
    fn submit_limit_order(
        &mut self,
        _meta: CommandMeta,
        _order: &NewLimitOrder,
    ) -> Result<SubmitReport, CommandError> {
        todo!()
    }

    /// Submit a pegged order
    fn submit_pegged_order(
        &mut self,
        _meta: CommandMeta,
        _order: &NewPeggedOrder,
    ) -> Result<SubmitReport, CommandError> {
        todo!()
    }
}
