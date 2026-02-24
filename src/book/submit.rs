use crate::{book::OrderBook, commands::*, execution::*};

use serde::{Deserialize, Serialize};

impl<E: Clone + Copy + Eq + Serialize + for<'de> Deserialize<'de> + core::fmt::Debug> OrderBook<E> {
    /// Execute a submit command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_submit(
        &mut self,
        submit_cmd: SubmitCmd<E>,
    ) -> Result<SubmitReport, ExecutionError> {
        todo!()
    }
}
