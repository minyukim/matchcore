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

        Ok(self.submit_validated_limit_order(OrderId::from(meta.sequence_number), order))
    }

    /// Submit a validated limit order
    pub(super) fn submit_validated_limit_order(
        &mut self,
        id: OrderId,
        order: &LimitOrder,
    ) -> SubmitReport {
        if self.has_crossable_order(order.side(), order.price()) {
            self.submit_crossable_order(id, order)
        } else {
            self.submit_non_crossable_order(id, order)
        }
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

        Ok(self.submit_validated_pegged_order(OrderId::from(meta.sequence_number), order))
    }

    /// Submit a validated pegged order
    pub(super) fn submit_validated_pegged_order(
        &mut self,
        id: OrderId,
        order: &PeggedOrder,
    ) -> SubmitReport {
        match order.peg_reference() {
            PegReference::Primary => self.submit_primary_pegged_order(id, order),
            PegReference::Market => self.submit_market_pegged_order(id, order),
            PegReference::MidPrice => self.submit_mid_price_pegged_order(id, order),
        }
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

#[cfg(test)]
mod tests_submit_market_order {
    use super::*;

    fn submit(book: &mut OrderBook, seq: u64, ts: u64, order: MarketOrder) -> CommandOutcome {
        book.execute_submit(
            CommandMeta {
                sequence_number: SequenceNumber(seq),
                timestamp: Timestamp(ts),
            },
            &SubmitCmd {
                order: NewOrder::Market(order),
            },
        )
    }

    fn unwrap_submit_report(outcome: CommandOutcome) -> SubmitReport {
        match outcome {
            CommandOutcome::Applied(CommandReport::Submit(report)) => report,
            other => panic!("expected applied submit, got: {other:?}"),
        }
    }

    #[test]
    fn cancel_order_on_empty_opposite_side() {
        let mut book = OrderBook::new("TEST");

        let report = unwrap_submit_report(submit(
            &mut book,
            0,
            0,
            MarketOrder::new(Quantity(10), Side::Buy, false),
        ));

        assert_eq!(
            report.submitted_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                available: Quantity(0)
            })
        );
        assert!(report.submitted_order().match_result().is_none());
    }

    #[test]
    fn market_to_limit_converts_remaining_to_limit_at_last_trade() {
        let mut book = OrderBook::new("TEST");

        // Seed sell-side liquidity at 100 for 5 units.
        book.add_limit_order(
            OrderId(10),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_submit_report(submit(
            &mut book,
            0,
            0,
            MarketOrder::new(Quantity(10), Side::Buy, true),
        ));

        let submitted = report.submitted_order();
        assert_eq!(submitted.order_id(), OrderId(0));
        assert_eq!(
            submitted.match_result().unwrap().executed_quantity(),
            Quantity(5)
        );
        assert!(submitted.cancel_reason().is_none());
        assert_eq!(book.last_trade_price(), Some(Price(100)));

        // Remaining quantity should become a resting buy limit at last trade price.
        assert!(book.limit.orders.contains_key(&submitted.order_id()));
        let resting = book.limit.orders.get(&submitted.order_id()).unwrap();
        assert_eq!(resting.side(), Side::Buy);
        assert_eq!(resting.price(), Price(100));
        assert_eq!(resting.total_quantity(), Quantity(5));
    }
}

#[cfg(test)]
mod tests_submit_limit_order {
    use super::*;

    fn submit(book: &mut OrderBook, seq: u64, ts: u64, order: LimitOrder) -> CommandOutcome {
        book.execute_submit(
            CommandMeta {
                sequence_number: SequenceNumber(seq),
                timestamp: Timestamp(ts),
            },
            &SubmitCmd {
                order: NewOrder::Limit(order),
            },
        )
    }

    fn unwrap_submit_report(outcome: CommandOutcome) -> SubmitReport {
        match outcome {
            CommandOutcome::Applied(CommandReport::Submit(report)) => report,
            other => panic!("expected applied submit, got: {other:?}"),
        }
    }

    #[test]
    fn reject_expired_order() {
        let mut book = OrderBook::new("TEST");

        let outcome = submit(
            &mut book,
            0,
            1000,
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );

        match outcome {
            CommandOutcome::Rejected(RejectReason::CommandError(CommandError::Expired)) => {}
            other => panic!("expected expired rejection, got: {other:?}"),
        }
    }

    #[test]
    fn cancel_immediate_order_on_non_crossable() {
        let mut book = OrderBook::new("TEST");

        let report = unwrap_submit_report(submit(
            &mut book,
            0,
            0,
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Ioc),
            ),
        ));

        assert_eq!(
            report.submitted_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                available: Quantity(0)
            })
        );
        assert!(report.submitted_order().match_result().is_none());
        assert!(book.limit.orders.is_empty());
    }

    #[test]
    fn cancel_post_only_order_on_crossable() {
        let mut book = OrderBook::new("TEST");

        // Seed sell-side best ask at 100.
        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_submit_report(submit(
            &mut book,
            1,
            0,
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            ),
        ));

        assert_eq!(
            report.submitted_order().cancel_reason(),
            Some(&CancelReason::PostOnlyWouldTake)
        );
        assert!(report.submitted_order().match_result().is_none());
    }

    #[test]
    fn cancel_fok_order_on_crossable_insufficient_liquidity() {
        let mut book = OrderBook::new("TEST");

        // Seed sell-side liquidity at 100 for 5.
        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_submit_report(submit(
            &mut book,
            1,
            0,
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Fok),
            ),
        ));

        assert_eq!(
            report.submitted_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                available: Quantity(5)
            })
        );
        assert!(report.submitted_order().match_result().is_none());
        assert_eq!(book.last_trade_price(), None);
    }

    #[test]
    fn cancel_ioc_order_on_crossable_after_partial_match() {
        let mut book = OrderBook::new("TEST");

        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_submit_report(submit(
            &mut book,
            1,
            0,
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Ioc),
            ),
        ));

        let submitted = report.submitted_order();
        assert_eq!(
            submitted.match_result().unwrap().executed_quantity(),
            Quantity(5)
        );
        assert_eq!(
            submitted.cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                available: Quantity(5)
            })
        );
        assert!(!book.limit.orders.contains_key(&submitted.order_id()));
    }

    #[test]
    fn rest_remaining_order_on_crossable_after_partial_match() {
        let mut book = OrderBook::new("TEST");

        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_submit_report(submit(
            &mut book,
            1,
            0,
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        ));

        let submitted = report.submitted_order();
        assert_eq!(
            submitted.match_result().unwrap().executed_quantity(),
            Quantity(5)
        );
        assert!(submitted.cancel_reason().is_none());
        assert_eq!(book.last_trade_price(), Some(Price(100)));

        let resting = book.limit.orders.get(&submitted.order_id()).unwrap();
        assert_eq!(resting.side(), Side::Buy);
        assert_eq!(resting.price(), Price(100));
        assert_eq!(resting.total_quantity(), Quantity(5));
    }
}

#[cfg(test)]
mod tests_submit_pegged_order {
    use super::*;

    fn submit(book: &mut OrderBook, seq: u64, ts: u64, order: PeggedOrder) -> CommandOutcome {
        book.execute_submit(
            CommandMeta {
                sequence_number: SequenceNumber(seq),
                timestamp: Timestamp(ts),
            },
            &SubmitCmd {
                order: NewOrder::Pegged(order),
            },
        )
    }

    fn unwrap_submit_report(outcome: CommandOutcome) -> SubmitReport {
        match outcome {
            CommandOutcome::Applied(CommandReport::Submit(report)) => report,
            other => panic!("expected applied submit, got: {other:?}"),
        }
    }

    #[test]
    fn reject_expired_order() {
        let mut book = OrderBook::new("TEST");

        let outcome = submit(
            &mut book,
            0,
            1000,
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );

        match outcome {
            CommandOutcome::Rejected(RejectReason::CommandError(CommandError::Expired)) => {}
            other => panic!("expected expired rejection, got: {other:?}"),
        }
    }

    #[test]
    fn add_primary_pegged_order_to_book() {
        let mut book = OrderBook::new("TEST");

        let report = unwrap_submit_report(submit(
            &mut book,
            0,
            0,
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        ));

        let id = report.submitted_order().order_id();
        assert!(book.pegged.orders.contains_key(&id));
        assert!(report.submitted_order().match_result().is_none());
        assert!(report.submitted_order().cancel_reason().is_none());
    }

    #[test]
    fn cancel_immediate_order_on_empty_opposite_side() {
        let mut book = OrderBook::new("TEST");

        let report = unwrap_submit_report(submit(
            &mut book,
            0,
            0,
            PeggedOrder::new(
                PegReference::Market,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Ioc),
            ),
        ));

        assert_eq!(
            report.submitted_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                available: Quantity(0)
            })
        );
        assert!(book.pegged.orders.is_empty());
    }

    #[test]
    fn rest_remaining_market_pegged_order_after_partial_match() {
        let mut book = OrderBook::new("TEST");

        // Provide sell-side liquidity so the market-pegged buy can match.
        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_submit_report(submit(
            &mut book,
            0,
            0,
            PeggedOrder::new(
                PegReference::Market,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        ));

        let submitted = report.submitted_order();
        assert_eq!(
            submitted.match_result().unwrap().executed_quantity(),
            Quantity(5)
        );
        assert!(submitted.cancel_reason().is_none());

        let resting = book.pegged.orders.get(&submitted.order_id()).unwrap();
        assert_eq!(resting.peg_reference(), PegReference::Market);
        assert_eq!(resting.side(), Side::Buy);
        assert_eq!(resting.quantity(), Quantity(5));
    }
}
