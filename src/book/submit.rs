use crate::{book::OrderBook, command::*, execution::*};

use serde::{Deserialize, Serialize};

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> OrderBook<E> {
    /// Execute a submit command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_submit(
        &mut self,
        meta: CommandMeta,
        cmd: &SubmitCmd<E>,
    ) -> Result<SubmitReport, ExecutionError> {
        match &cmd.order {
            NewOrder::Market(order) => self.submit_market_order(meta, order),
            NewOrder::Limit(order) => self.submit_limit_order(meta, order),
            NewOrder::Pegged(order) => self.submit_pegged_order(meta, order),
        }
    }

    /// Submit a market order
    fn submit_market_order(
        &mut self,
        _meta: CommandMeta,
        _order: &NewMarketOrder<E>,
    ) -> Result<SubmitReport, ExecutionError> {
        todo!()
    }

    /// Submit a limit order
    fn submit_limit_order(
        &mut self,
        _meta: CommandMeta,
        _order: &NewLimitOrder<E>,
    ) -> Result<SubmitReport, ExecutionError> {
        todo!()
    }

    /// Submit a pegged order
    fn submit_pegged_order(
        &mut self,
        _meta: CommandMeta,
        _order: &NewPeggedOrder<E>,
    ) -> Result<SubmitReport, ExecutionError> {
        todo!()
    }
}
