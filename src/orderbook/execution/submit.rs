use super::OrderBook;
use crate::{command::*, orders::*, outcome::*, types::*};

impl OrderBook {
    /// Execute a submit command against the order book and return the execution outcome
    pub(super) fn execute_submit(&mut self, meta: CommandMeta, cmd: &SubmitCmd) -> CommandOutcome {
        let (was_bid_empty, was_ask_empty) = (
            self.is_side_empty(Side::Buy),
            self.is_side_empty(Side::Sell),
        );

        let result = match &cmd.order {
            NewOrder::Market(order) => self.submit_market_order(meta.sequence_number, order),
            NewOrder::Limit(order) => self.submit_limit_order(meta, order),
            NewOrder::Pegged(order) => self.submit_pegged_order(meta, order),
        };
        let mut effects = match result {
            Ok(effects) => effects,
            Err(failure) => return CommandOutcome::Rejected(failure),
        };

        loop {
            let bid_became_non_empty = was_bid_empty && !self.is_side_empty(Side::Buy);
            let ask_became_non_empty = was_ask_empty && !self.is_side_empty(Side::Sell);

            let Some(order_outcome) = self.match_market_pegged_order(
                meta.sequence_number,
                bid_became_non_empty,
                ask_became_non_empty,
            ) else {
                break;
            };
            effects.add_triggered_order(order_outcome);
        }

        CommandOutcome::Applied(CommandReport::Submit(effects))
    }

    /// Submit a market order
    fn submit_market_order(
        &mut self,
        sequence_number: SequenceNumber,
        order: &MarketOrder,
    ) -> Result<CommandEffects, CommandFailure> {
        order.validate().map_err(CommandFailure::InvalidCommand)?;

        Ok(CommandEffects::new(self.submit_validated_market_order(
            sequence_number,
            OrderId::from(sequence_number),
            order,
        )))
    }

    /// Submit a validated market order
    pub(super) fn submit_validated_market_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: &MarketOrder,
    ) -> OrderOutcome {
        let mut outcome = OrderOutcome::new(id);

        if self.is_side_empty(order.side().opposite()) {
            outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                requested: order.quantity(),
                available: Quantity(0),
            });
            return outcome;
        }

        let result = self.match_order(sequence_number, order.side(), None, order.quantity());
        let executed_quantity = result.executed_quantity();
        outcome.set_match_result(result);

        let remaining_quantity = order.quantity() - executed_quantity;
        if remaining_quantity.is_zero() {
            return outcome;
        }

        // If the order is a market to limit order and there is a remaining quantity,
        // convert it to a limit order at the last trade price
        if order.market_to_limit() {
            // The last trade price is guaranteed to exist because the order was matched
            let price = self.last_trade_price.unwrap();

            self.add_limit_order(
                sequence_number,
                id,
                LimitOrder::new(
                    price,
                    QuantityPolicy::Standard {
                        quantity: remaining_quantity,
                    },
                    OrderFlags::new(order.side(), false, TimeInForce::Gtc),
                ),
            );

            return outcome;
        }

        outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
            requested: order.quantity(),
            available: executed_quantity,
        });

        outcome
    }

    /// Submit a limit order
    fn submit_limit_order(
        &mut self,
        meta: CommandMeta,
        order: &LimitOrder,
    ) -> Result<CommandEffects, CommandFailure> {
        order.validate().map_err(CommandFailure::InvalidCommand)?;

        if order.is_expired(meta.timestamp) {
            return Err(CommandFailure::InvalidCommand(CommandError::Expired));
        }

        Ok(self.submit_validated_limit_order(
            meta.sequence_number,
            OrderId::from(meta.sequence_number),
            order,
        ))
    }

    /// Submit a validated limit order
    pub(super) fn submit_validated_limit_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: &LimitOrder,
    ) -> CommandEffects {
        if self.has_crossable_order(order.side(), order.price()) {
            self.submit_crossable_order(sequence_number, id, order)
        } else {
            self.submit_non_crossable_order(sequence_number, id, order)
        }
    }

    /// Submit a crossable order
    fn submit_crossable_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: &LimitOrder,
    ) -> CommandEffects {
        let mut outcome = OrderOutcome::new(id);

        if order.post_only() {
            outcome.set_cancel_reason(CancelReason::PostOnlyWouldTake);
            return CommandEffects::new(outcome);
        }

        if order.time_in_force() == TimeInForce::Fok {
            let executable_quantity = self.max_executable_quantity_with_limit_price_unchecked(
                order.side(),
                order.price(),
                order.total_quantity(),
            );
            if executable_quantity < order.total_quantity() {
                outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                    requested: order.total_quantity(),
                    available: executable_quantity,
                });
                return CommandEffects::new(outcome);
            }
        }

        let result = self.match_order(sequence_number, order.side(), None, order.total_quantity());
        let executed_quantity = result.executed_quantity();
        outcome.set_match_result(result);

        let remaining_quantity = order.total_quantity() - executed_quantity;
        if remaining_quantity.is_zero() {
            return CommandEffects::new(outcome);
        }

        if order.time_in_force() == TimeInForce::Ioc {
            outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                requested: order.total_quantity(),
                available: executed_quantity,
            });
            return CommandEffects::new(outcome);
        }

        let quantity_policy = order
            .quantity_policy()
            .with_remaining_quantity(remaining_quantity);

        self.add_limit_order(
            sequence_number,
            id,
            LimitOrder::new(order.price(), quantity_policy, order.flags().clone()),
        );

        CommandEffects::new(outcome)
    }

    /// Submit a non-crossable order
    fn submit_non_crossable_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: &LimitOrder,
    ) -> CommandEffects {
        let mut outcome = OrderOutcome::new(id);

        if order.is_immediate() {
            outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                requested: order.total_quantity(),
                available: Quantity(0),
            });
            return CommandEffects::new(outcome);
        }

        self.add_limit_order(sequence_number, id, order.clone());

        CommandEffects::new(outcome)
    }

    /// Submit a pegged order
    fn submit_pegged_order(
        &mut self,
        meta: CommandMeta,
        order: &PeggedOrder,
    ) -> Result<CommandEffects, CommandFailure> {
        order.validate().map_err(CommandFailure::InvalidCommand)?;

        if order.is_expired(meta.timestamp) {
            return Err(CommandFailure::InvalidCommand(CommandError::Expired));
        }

        Ok(self.submit_validated_pegged_order(
            meta.sequence_number,
            OrderId::from(meta.sequence_number),
            order,
        ))
    }

    /// Submit a validated pegged order
    pub(super) fn submit_validated_pegged_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: &PeggedOrder,
    ) -> CommandEffects {
        match order.peg_reference() {
            PegReference::Primary => self.submit_primary_pegged_order(sequence_number, id, order),
            PegReference::Market => self.submit_market_pegged_order(sequence_number, id, order),
            PegReference::MidPrice => {
                self.submit_mid_price_pegged_order(sequence_number, id, order)
            }
        }
    }

    /// Submit a primary pegged order
    fn submit_primary_pegged_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: &PeggedOrder,
    ) -> CommandEffects {
        self.submit_unmarketable_pegged_order(sequence_number, id, order)
    }

    /// Submit a market pegged order
    fn submit_market_pegged_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: &PeggedOrder,
    ) -> CommandEffects {
        let mut outcome = OrderOutcome::new(id);

        if self.is_side_empty(order.side().opposite()) {
            if order.is_immediate() {
                outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                    requested: order.quantity(),
                    available: Quantity(0),
                });
                return CommandEffects::new(outcome);
            }

            self.add_pegged_order(sequence_number, id, order.clone());

            return CommandEffects::new(outcome);
        }

        if order.time_in_force() == TimeInForce::Fok {
            let executable_quantity =
                self.max_executable_quantity_unchecked(order.side(), order.quantity());
            if executable_quantity < order.quantity() {
                outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                    requested: order.quantity(),
                    available: executable_quantity,
                });
                return CommandEffects::new(outcome);
            }
        }

        let result = self.match_order(sequence_number, order.side(), None, order.quantity());
        let executed_quantity = result.executed_quantity();
        outcome.set_match_result(result);

        let remaining_quantity = order.quantity() - executed_quantity;
        if remaining_quantity.is_zero() {
            return CommandEffects::new(outcome);
        }

        if order.time_in_force() == TimeInForce::Ioc {
            outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                requested: order.quantity(),
                available: executed_quantity,
            });
            return CommandEffects::new(outcome);
        }

        self.add_pegged_order(
            sequence_number,
            id,
            PeggedOrder::new(
                order.peg_reference(),
                remaining_quantity,
                order.flags().clone(),
            ),
        );

        CommandEffects::new(outcome)
    }

    /// Submit a mid price pegged order
    fn submit_mid_price_pegged_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: &PeggedOrder,
    ) -> CommandEffects {
        self.submit_unmarketable_pegged_order(sequence_number, id, order)
    }

    /// Submit an unmarketable pegged order
    fn submit_unmarketable_pegged_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: &PeggedOrder,
    ) -> CommandEffects {
        self.add_pegged_order(sequence_number, id, order.clone());

        CommandEffects::new(OrderOutcome::new(id))
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

    fn unwrap_submit_effects(outcome: CommandOutcome) -> CommandEffects {
        match outcome {
            CommandOutcome::Applied(CommandReport::Submit(effects)) => effects,
            other => panic!("expected applied submit, got: {other:?}"),
        }
    }

    #[test]
    fn cancel_order_on_empty_opposite_side() {
        let mut book = OrderBook::new("TEST");

        let effects = unwrap_submit_effects(submit(
            &mut book,
            0,
            0,
            MarketOrder::new(Quantity(10), Side::Buy, false),
        ));

        assert_eq!(
            effects.target_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                requested: Quantity(10),
                available: Quantity(0)
            })
        );
        assert!(effects.target_order().match_result().is_none());
    }

    #[test]
    fn market_to_limit_converts_remaining_to_limit_at_last_trade() {
        let mut book = OrderBook::new("TEST");

        // Seed sell-side liquidity at 100 for 5 units.
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_submit_effects(submit(
            &mut book,
            1,
            1,
            MarketOrder::new(Quantity(10), Side::Buy, true),
        ));

        let submitted = effects.target_order();
        assert_eq!(submitted.order_id(), OrderId(1));
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

    fn unwrap_submit_effects(outcome: CommandOutcome) -> CommandEffects {
        match outcome {
            CommandOutcome::Applied(CommandReport::Submit(effects)) => effects,
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
            CommandOutcome::Rejected(CommandFailure::InvalidCommand(CommandError::Expired)) => {}
            other => panic!("expected expired rejection, got: {other:?}"),
        }
    }

    #[test]
    fn cancel_immediate_order_on_non_crossable() {
        let mut book = OrderBook::new("TEST");

        let effects = unwrap_submit_effects(submit(
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
            effects.target_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                requested: Quantity(10),
                available: Quantity(0)
            })
        );
        assert!(effects.target_order().match_result().is_none());
        assert!(book.limit.orders.is_empty());
    }

    #[test]
    fn cancel_post_only_order_on_crossable() {
        let mut book = OrderBook::new("TEST");

        // Seed sell-side best ask at 100.
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_submit_effects(submit(
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
            effects.target_order().cancel_reason(),
            Some(&CancelReason::PostOnlyWouldTake)
        );
        assert!(effects.target_order().match_result().is_none());
    }

    #[test]
    fn cancel_fok_order_on_crossable_insufficient_liquidity() {
        let mut book = OrderBook::new("TEST");

        // Seed sell-side liquidity at 100 for 5.
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_submit_effects(submit(
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
            effects.target_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                requested: Quantity(10),
                available: Quantity(5)
            })
        );
        assert!(effects.target_order().match_result().is_none());
        assert_eq!(book.last_trade_price(), None);
    }

    #[test]
    fn cancel_ioc_order_on_crossable_after_partial_match() {
        let mut book = OrderBook::new("TEST");

        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_submit_effects(submit(
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

        let submitted = effects.target_order();
        assert_eq!(
            submitted.match_result().unwrap().executed_quantity(),
            Quantity(5)
        );
        assert_eq!(
            submitted.cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                requested: Quantity(10),
                available: Quantity(5)
            })
        );
        assert!(!book.limit.orders.contains_key(&submitted.order_id()));
    }

    #[test]
    fn rest_remaining_order_on_crossable_after_partial_match() {
        let mut book = OrderBook::new("TEST");

        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_submit_effects(submit(
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

        let submitted = effects.target_order();
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

    fn unwrap_submit_effects(outcome: CommandOutcome) -> CommandEffects {
        match outcome {
            CommandOutcome::Applied(CommandReport::Submit(effects)) => effects,
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
            CommandOutcome::Rejected(CommandFailure::InvalidCommand(CommandError::Expired)) => {}
            other => panic!("expected expired rejection, got: {other:?}"),
        }
    }

    #[test]
    fn add_primary_pegged_order_to_book() {
        let mut book = OrderBook::new("TEST");

        let effects = unwrap_submit_effects(submit(
            &mut book,
            0,
            0,
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        ));

        let id = effects.target_order().order_id();
        assert!(book.pegged.orders.contains_key(&id));
        assert!(effects.target_order().match_result().is_none());
        assert!(effects.target_order().cancel_reason().is_none());
    }

    #[test]
    fn cancel_immediate_order_on_empty_opposite_side() {
        let mut book = OrderBook::new("TEST");

        let effects = unwrap_submit_effects(submit(
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
            effects.target_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                requested: Quantity(10),
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
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_submit_effects(submit(
            &mut book,
            1,
            0,
            PeggedOrder::new(
                PegReference::Market,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        ));

        let submitted = effects.target_order();
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
