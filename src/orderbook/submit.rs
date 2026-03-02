use crate::{
    command::*,
    orderbook::{OrderBook, PriceLevel},
    orders::*,
    report::*,
    types::*,
};

use std::collections::btree_map::Entry;

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

            // Remaining means there is no price level at this price, so create a new one
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
            self.insert_price_level(spec.side(), price, price_level);

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

        if spec.time_in_force() == TimeInForce::Fok {
            let executable_quantity = self.max_executable_quantity_unchecked(
                spec.side(),
                spec.price(),
                spec.total_quantity(),
            );
            if executable_quantity < spec.total_quantity() {
                return Err(RejectReason::InsufficientLiquidity {
                    requested_quantity: spec.total_quantity(),
                    available_quantity: executable_quantity,
                });
            }
        }

        let order_id = meta.sequence_number;

        let result = self.match_order(spec.side(), None, spec.total_quantity(), meta.timestamp);

        let executed_quantity = result.executed_quantity();
        let remaining_quantity = spec.total_quantity() - executed_quantity;
        if remaining_quantity == 0 {
            return Ok(SubmitReport::new(
                OrderProcessingResult::new(order_id).with_match_result(result),
            ));
        }

        let quantity_policy = match spec.quantity_policy() {
            QuantityPolicy::Standard { .. } => QuantityPolicy::Standard {
                quantity: remaining_quantity,
            },
            QuantityPolicy::Iceberg {
                replenish_quantity, ..
            } => {
                let visible_quantity = ((remaining_quantity - 1) % replenish_quantity) + 1;

                QuantityPolicy::Iceberg {
                    visible_quantity,
                    hidden_quantity: remaining_quantity - visible_quantity,
                    replenish_quantity,
                }
            }
        };

        // Remaining means there is no price level at this price, so create a new one
        let mut price_level = PriceLevel::new();
        price_level.push(
            &mut self.limit_orders,
            LimitOrder::new(
                order_id,
                LimitOrderSpec::new(spec.price(), quantity_policy, spec.flags().clone()),
            ),
        );
        self.insert_price_level(spec.side(), spec.price(), price_level);

        let triggered_orders =
            self.trigger_opposite_side_takers(spec.side().opposite(), meta.timestamp);

        Ok(
            SubmitReport::new(OrderProcessingResult::new(order_id).with_match_result(result))
                .with_triggered_orders(triggered_orders),
        )
    }

    fn submit_non_crossable_order(
        &mut self,
        meta: CommandMeta,
        spec: &LimitOrderSpec,
    ) -> Result<SubmitReport, RejectReason> {
        if spec.is_immediate() {
            return Err(RejectReason::NoLiquidity);
        }

        let order_id = meta.sequence_number;

        let orders = &mut self.limit_orders;

        let levels = match spec.side() {
            Side::Buy => &mut self.limit_bid_levels,
            Side::Sell => &mut self.limit_ask_levels,
        };

        match levels.entry(spec.price()) {
            Entry::Occupied(mut e) => {
                e.get_mut()
                    .push(orders, LimitOrder::new(order_id, spec.clone()));
            }
            Entry::Vacant(e) => {
                let mut price_level = PriceLevel::new();
                price_level.push(orders, LimitOrder::new(order_id, spec.clone()));
                e.insert(price_level);
            }
        }

        Ok(SubmitReport::new(OrderProcessingResult::new(order_id)))
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
