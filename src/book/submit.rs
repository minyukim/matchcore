use crate::{book::OrderBook, commands::*, execution::*};

use serde::{Deserialize, Serialize};

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> OrderBook<E> {
    /// Execute a submit command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_submit(
        &mut self,
        cmd: &SubmitCmd<E>,
    ) -> Result<SubmitReport, ExecutionError> {
        match &cmd.order {
            NewOrder::Market(order) => self.submit_market_order(order),
            NewOrder::Limit(order) => self.submit_limit_order(order),
            NewOrder::Pegged(order) => self.submit_pegged_order(order),
        }
    }

    /// Submit a market order
    fn submit_market_order(
        &mut self,
        _order: &NewMarketOrder<E>,
    ) -> Result<SubmitReport, ExecutionError> {
        todo!()
    }

    /// Submit a limit order
    fn submit_limit_order(
        &mut self,
        _order: &NewLimitOrder<E>,
    ) -> Result<SubmitReport, ExecutionError> {
        todo!()
    }

    /// Submit a pegged order
    fn submit_pegged_order(
        &mut self,
        _order: &NewPeggedOrder<E>,
    ) -> Result<SubmitReport, ExecutionError> {
        todo!()
    }
}
