use crate::{
    command::*,
    orderbook::{OrderBook, PriceLevel},
    orders::*,
    report::*,
    types::*,
};

impl OrderBook {
    /// Execute a submit command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_submit(&mut self, meta: CommandMeta, cmd: &SubmitCmd) -> CommandOutcome {
        let result = match &cmd.order {
            NewOrder::Market(spec) => self.submit_market_order(meta, spec),
            NewOrder::Limit(spec) => self.submit_limit_order(meta, spec),
            NewOrder::Pegged(spec) => self.submit_pegged_order(meta, spec),
        };

        match result {
            Ok(report) => CommandOutcome::Applied(CommandReport::Submit(report)),
            Err(reason) => CommandOutcome::Rejected(reason),
        }
    }

    /// Submit a market order
    fn submit_market_order(
        &mut self,
        meta: CommandMeta,
        spec: &MarketOrderSpec,
    ) -> Result<SubmitReport, RejectReason> {
        spec.validate().map_err(RejectReason::CommandError)?;

        if self.is_side_empty(spec.side().opposite()) {
            return Err(RejectReason::NoLiquidity);
        }

        let order_id = meta.sequence_number;

        let result = self.match_order(spec.side(), None, spec.quantity(), meta.timestamp);

        let executed_quantity = result.executed_quantity();
        let remaining_quantity = spec.quantity() - executed_quantity;
        if remaining_quantity == 0 {
            return Ok(SubmitReport::new(
                OrderProcessingResult::new(order_id).with_match_result(result),
            ));
        }

        // If the order is a market to limit order and there is a remaining quantity,
        // convert it to a limit order at the last trade price
        if spec.market_to_limit() {
            // The last trade price is guaranteed to exist because the order was matched
            let price = self.last_trade_price.unwrap();

            let mut price_level = PriceLevel::new();
            price_level.push(
                &mut self.limit_orders,
                LimitOrder::new(
                    order_id,
                    LimitOrderSpec::new(
                        price,
                        QuantityPolicy::Standard {
                            quantity: remaining_quantity,
                        },
                        OrderFlags::new(spec.side(), false, TimeInForce::Gtc),
                    ),
                ),
            );

            let price_levels = match spec.side() {
                Side::Buy => &mut self.limit_bid_levels,
                Side::Sell => &mut self.limit_ask_levels,
            };
            price_levels.insert(price, price_level);

            let triggered_orders =
                self.trigger_opposite_side_takers(spec.side().opposite(), meta.timestamp);

            return Ok(SubmitReport::new(
                OrderProcessingResult::new(order_id).with_match_result(result),
            )
            .with_triggered_orders(triggered_orders));
        }

        Ok(SubmitReport::new(
            OrderProcessingResult::new(order_id)
                .with_match_result(result)
                .with_cancel_reason(CancelReason::InsufficientLiquidity {
                    requested_quantity: spec.quantity(),
                    available_quantity: executed_quantity,
                }),
        ))
    }

    /// Submit a limit order
    fn submit_limit_order(
        &mut self,
        meta: CommandMeta,
        spec: &LimitOrderSpec,
    ) -> Result<SubmitReport, RejectReason> {
        spec.validate().map_err(RejectReason::CommandError)?;

        if spec.is_expired(meta.timestamp) {
            return Err(RejectReason::CommandError(CommandError::Expired));
        }

        if self.has_crossable_order(spec.side(), spec.price()) {
            self.submit_crossable_order(meta, spec)
        } else {
            self.submit_non_crossable_order(meta, spec)
        }
    }

    fn submit_crossable_order(
        &mut self,
        meta: CommandMeta,
        spec: &LimitOrderSpec,
    ) -> Result<SubmitReport, RejectReason> {
        if spec.post_only() {
            return Err(RejectReason::PostOnlyWouldTake);
        }

        todo!()
    }

    fn submit_non_crossable_order(
        &mut self,
        meta: CommandMeta,
        spec: &LimitOrderSpec,
    ) -> Result<SubmitReport, RejectReason> {
        if spec.is_immediate() {
            return Err(RejectReason::NoLiquidity);
        }

        todo!()
    }

    /// Submit a pegged order
    fn submit_pegged_order(
        &mut self,
        _meta: CommandMeta,
        _spec: &PeggedOrderSpec,
    ) -> Result<SubmitReport, RejectReason> {
        todo!()
    }
}
