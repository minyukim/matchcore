use super::OrderBook;
use crate::{command::*, orders::*, report::*, types::*};

impl OrderBook {
    /// Execute a submit command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_submit(&mut self, meta: CommandMeta, cmd: &SubmitCmd) -> CommandOutcome {
        let result = match &cmd.order {
            NewOrder::Market(order) => self.submit_market_order(meta.sequence_number, order),
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
        sequence_number: SequenceNumber,
        order: &MarketOrder,
    ) -> Result<SubmitReport, RejectReason> {
        order.validate().map_err(RejectReason::CommandError)?;

        let order_id = OrderId::from(sequence_number);

        if self.is_side_empty(order.side().opposite()) {
            return Ok(SubmitReport::new(
                OrderProcessingResult::new(order_id).with_cancel_reason(
                    CancelReason::InsufficientLiquidity {
                        available: Quantity(0),
                    },
                ),
            ));
        }

        let result = self.match_order(order.side(), None, order.quantity());

        let executed_quantity = result.executed_quantity();
        let remaining_quantity = order.quantity() - executed_quantity;
        if remaining_quantity.is_zero() {
            return Ok(SubmitReport::new(
                OrderProcessingResult::new(order_id).with_match_result(result),
            ));
        }

        // If the order is a market to limit order and there is a remaining quantity,
        // convert it to a limit order at the last trade price
        if order.market_to_limit() {
            // The last trade price is guaranteed to exist because the order was matched
            let price = self.last_trade_price.unwrap();

            self.add_limit_order(
                order_id,
                LimitOrder::new(
                    price,
                    QuantityPolicy::Standard {
                        quantity: remaining_quantity,
                    },
                    OrderFlags::new(order.side(), false, TimeInForce::Gtc),
                ),
            );

            let triggered_orders = self.trigger_opposite_side_takers(order.side().opposite());

            return Ok(SubmitReport::new(
                OrderProcessingResult::new(order_id).with_match_result(result),
            )
            .with_triggered_orders(triggered_orders));
        }

        Ok(SubmitReport::new(
            OrderProcessingResult::new(order_id)
                .with_match_result(result)
                .with_cancel_reason(CancelReason::InsufficientLiquidity {
                    available: executed_quantity,
                }),
        ))
    }

    /// Submit a limit order
    fn submit_limit_order(
        &mut self,
        meta: CommandMeta,
        order: &LimitOrder,
    ) -> Result<SubmitReport, RejectReason> {
        order.validate().map_err(RejectReason::CommandError)?;

        if order.is_expired(meta.timestamp) {
            return Err(RejectReason::CommandError(CommandError::Expired));
        }

        let order_id = OrderId::from(meta.sequence_number);

        Ok(if self.has_crossable_order(order.side(), order.price()) {
            self.submit_crossable_order(order_id, order)
        } else {
            self.submit_non_crossable_order(order_id, order)
        })
    }

    /// Submit a crossable order
    fn submit_crossable_order(&mut self, id: OrderId, order: &LimitOrder) -> SubmitReport {
        if order.post_only() {
            return SubmitReport::new(
                OrderProcessingResult::new(id).with_cancel_reason(CancelReason::PostOnlyWouldTake),
            );
        }

        if order.time_in_force() == TimeInForce::Fok {
            let executable_quantity = self.max_executable_quantity_with_limit_price_unchecked(
                order.side(),
                order.price(),
                order.total_quantity(),
            );
            if executable_quantity < order.total_quantity() {
                return SubmitReport::new(OrderProcessingResult::new(id).with_cancel_reason(
                    CancelReason::InsufficientLiquidity {
                        available: executable_quantity,
                    },
                ));
            }
        }

        let result = self.match_order(order.side(), None, order.total_quantity());

        let executed_quantity = result.executed_quantity();
        let remaining_quantity = order.total_quantity() - executed_quantity;
        if remaining_quantity.is_zero() {
            return SubmitReport::new(OrderProcessingResult::new(id).with_match_result(result));
        }

        if order.time_in_force() == TimeInForce::Ioc {
            return SubmitReport::new(
                OrderProcessingResult::new(id)
                    .with_match_result(result)
                    .with_cancel_reason(CancelReason::InsufficientLiquidity {
                        available: executed_quantity,
                    }),
            );
        }

        let quantity_policy = match order.quantity_policy() {
            QuantityPolicy::Standard { .. } => QuantityPolicy::Standard {
                quantity: remaining_quantity,
            },
            QuantityPolicy::Iceberg {
                replenish_quantity, ..
            } => {
                let visible_quantity =
                    Quantity(((remaining_quantity.0 - 1) % replenish_quantity.0) + 1);

                QuantityPolicy::Iceberg {
                    visible_quantity,
                    hidden_quantity: remaining_quantity - visible_quantity,
                    replenish_quantity,
                }
            }
        };

        self.add_limit_order(
            id,
            LimitOrder::new(order.price(), quantity_policy, order.flags().clone()),
        );

        let triggered_orders = self.trigger_opposite_side_takers(order.side().opposite());

        SubmitReport::new(OrderProcessingResult::new(id).with_match_result(result))
            .with_triggered_orders(triggered_orders)
    }

    /// Submit a non-crossable order
    fn submit_non_crossable_order(&mut self, id: OrderId, order: &LimitOrder) -> SubmitReport {
        if order.is_immediate() {
            return SubmitReport::new(OrderProcessingResult::new(id).with_cancel_reason(
                CancelReason::InsufficientLiquidity {
                    available: Quantity(0),
                },
            ));
        }

        self.add_limit_order(id, order.clone());

        let triggered_orders = self.trigger_opposite_side_takers(order.side().opposite());

        SubmitReport::new(OrderProcessingResult::new(id)).with_triggered_orders(triggered_orders)
    }

    /// Submit a pegged order
    fn submit_pegged_order(
        &mut self,
        meta: CommandMeta,
        order: &PeggedOrder,
    ) -> Result<SubmitReport, RejectReason> {
        order.validate().map_err(RejectReason::CommandError)?;

        if order.is_expired(meta.timestamp) {
            return Err(RejectReason::CommandError(CommandError::Expired));
        }

        let order_id = OrderId::from(meta.sequence_number);

        Ok(match order.peg_reference() {
            PegReference::Primary => self.submit_primary_pegged_order(order_id, order),
            PegReference::Market => self.submit_market_pegged_order(order_id, order),
            PegReference::MidPrice => self.submit_mid_price_pegged_order(order_id, order),
        })
    }

    /// Submit a primary pegged order
    fn submit_primary_pegged_order(&mut self, id: OrderId, order: &PeggedOrder) -> SubmitReport {
        self.submit_unmarketable_pegged_order(id, order)
    }

    /// Submit a market pegged order
    fn submit_market_pegged_order(&mut self, id: OrderId, order: &PeggedOrder) -> SubmitReport {
        if self.is_side_empty(order.side().opposite()) {
            if order.is_immediate() {
                return SubmitReport::new(OrderProcessingResult::new(id).with_cancel_reason(
                    CancelReason::InsufficientLiquidity {
                        available: Quantity(0),
                    },
                ));
            }

            self.add_pegged_order(id, order.clone());

            return SubmitReport::new(OrderProcessingResult::new(id));
        }

        if order.time_in_force() == TimeInForce::Fok {
            let executable_quantity =
                self.max_executable_quantity_unchecked(order.side(), order.quantity());
            if executable_quantity < order.quantity() {
                return SubmitReport::new(OrderProcessingResult::new(id).with_cancel_reason(
                    CancelReason::InsufficientLiquidity {
                        available: executable_quantity,
                    },
                ));
            }
        }

        let result = self.match_order(order.side(), None, order.quantity());

        let executed_quantity = result.executed_quantity();
        let remaining_quantity = order.quantity() - executed_quantity;
        if remaining_quantity.is_zero() {
            return SubmitReport::new(OrderProcessingResult::new(id).with_match_result(result));
        }

        if order.time_in_force() == TimeInForce::Ioc {
            return SubmitReport::new(
                OrderProcessingResult::new(id)
                    .with_match_result(result)
                    .with_cancel_reason(CancelReason::InsufficientLiquidity {
                        available: executed_quantity,
                    }),
            );
        }

        self.add_pegged_order(
            id,
            PeggedOrder::new(
                order.peg_reference(),
                remaining_quantity,
                order.flags().clone(),
            ),
        );

        SubmitReport::new(OrderProcessingResult::new(id).with_match_result(result))
    }

    /// Submit a mid price pegged order
    fn submit_mid_price_pegged_order(&mut self, id: OrderId, order: &PeggedOrder) -> SubmitReport {
        self.submit_unmarketable_pegged_order(id, order)
    }

    /// Submit an unmarketable pegged order
    fn submit_unmarketable_pegged_order(
        &mut self,
        id: OrderId,
        order: &PeggedOrder,
    ) -> SubmitReport {
        self.add_pegged_order(id, order.clone());

        SubmitReport::new(OrderProcessingResult::new(id))
    }
}
