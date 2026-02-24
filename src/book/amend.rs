use crate::{book::OrderBook, commands::*, execution::*};

use serde::{Deserialize, Serialize};

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> OrderBook<E> {
    /// Execute an amend command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_amend(
        &mut self,
        amend_cmd: AmendCmd,
    ) -> Result<AmendReport, ExecutionError> {
        todo!()
    }
}
