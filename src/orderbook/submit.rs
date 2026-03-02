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
            NewOrder::Market(order) => self.submit_market_order(meta, order),
            NewOrder::Limit(order) => self.submit_limit_order(meta, order),
            NewOrder::Pegged(order) => self.submit_pegged_order(meta, order),
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
        order: &MarketOrderSpec,
    ) -> Result<SubmitReport, RejectReason> {
        order.validate().map_err(RejectReason::CommandError)?;

        if self.is_side_empty(order.side().opposite()) {
            return Err(RejectReason::NoLiquidity);
        }

        let order_id = meta.sequence_number;

        let result = self.match_order(order.side(), None, order.quantity(), meta.timestamp);

        let executed_quantity = result.executed_quantity();
        let remaining_quantity = order.quantity() - executed_quantity;
        if remaining_quantity == 0 {
            return Ok(SubmitReport::new(
                OrderProcessingResult::new(order_id).with_match_result(result),
            ));
        }

        // If the order is a market to limit order and there is a remaining quantity,
        // convert it to a limit order at the last trade price
        if order.market_to_limit() {
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
                        OrderFlags::new(order.side(), false, TimeInForce::Gtc),
                    ),
                ),
            );

            let price_levels = match order.side() {
                Side::Buy => &mut self.limit_bid_levels,
                Side::Sell => &mut self.limit_ask_levels,
            };
            price_levels.insert(price, price_level);

            let triggered_orders =
                self.trigger_opposite_side_takers(order.side().opposite(), meta.timestamp);

            return Ok(SubmitReport::new(
                OrderProcessingResult::new(order_id).with_match_result(result),
            )
            .with_triggered_orders(triggered_orders));
        }

        Ok(SubmitReport::new(
            OrderProcessingResult::new(order_id)
                .with_match_result(result)
                .with_cancel_reason(CancelReason::InsufficientLiquidity {
                    requested_quantity: order.quantity(),
                    available_quantity: executed_quantity,
                }),
        ))
    }

    /// Submit a limit order
    fn submit_limit_order(
        &mut self,
        _meta: CommandMeta,
        _order: &LimitOrderSpec,
    ) -> Result<SubmitReport, RejectReason> {
        todo!()
    }

    /// Submit a pegged order
    fn submit_pegged_order(
        &mut self,
        _meta: CommandMeta,
        _order: &PeggedOrderSpec,
    ) -> Result<SubmitReport, RejectReason> {
        todo!()
    }
}
