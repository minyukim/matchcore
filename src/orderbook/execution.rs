//! Execution of commands against the order book

mod amend;
mod cancel;
mod submit;
mod trigger;

use super::OrderBook;
use crate::{SequenceNumber, Timestamp, command::*, outcome::*};

use std::cmp::Reverse;

impl OrderBook {
    /// Execute a command against the order book
    /// Returns the execution report for the command
    pub fn execute(&mut self, cmd: &Command) -> CommandOutcome {
        if let Err(failure) = self.handle_command_meta(cmd.meta) {
            return CommandOutcome::Rejected(failure);
        }

        self.clean_up_expired_orders(cmd.meta);

        match &cmd.kind {
            CommandKind::Submit(submit_cmd) => self.execute_submit(cmd.meta, submit_cmd),
            CommandKind::Amend(amend_cmd) => self.execute_amend(cmd.meta, amend_cmd),
            CommandKind::Cancel(cancel_cmd) => {
                self.execute_cancel(cmd.meta.sequence_number, cancel_cmd)
            }
        }
    }

    /// Handle the command metadata
    /// Validates the command metadata, and updates the last sequence number and
    /// the last seen timestamp of the order book if the command is valid.
    fn handle_command_meta(&mut self, meta: CommandMeta) -> Result<(), CommandFailure> {
        self.validate_command_meta(meta)?;

        self.last_sequence_number = Some(meta.sequence_number);
        self.last_seen_timestamp = Some(meta.timestamp);

        Ok(())
    }

    /// Validate the command metadata
    fn validate_command_meta(&self, meta: CommandMeta) -> Result<(), CommandFailure> {
        self.validate_sequence_number(meta.sequence_number)?;
        self.validate_timestamp(meta.timestamp)?;
        Ok(())
    }

    /// Validate the sequence number of the command
    fn validate_sequence_number(
        &self,
        sequence_number: SequenceNumber,
    ) -> Result<(), CommandFailure> {
        let expected_sequence_number = match self.last_sequence_number {
            Some(last_sequence_number) => last_sequence_number.next(),
            None => SequenceNumber(0),
        };
        if sequence_number != expected_sequence_number {
            return Err(CommandFailure::InvalidSequenceNumber {
                expected_sequence_number,
                received_sequence_number: sequence_number,
            });
        }
        Ok(())
    }

    /// Validate the timestamp of the command
    fn validate_timestamp(&self, timestamp: Timestamp) -> Result<(), CommandFailure> {
        if let Some(last_seen_timestamp) = self.last_seen_timestamp
            && timestamp < last_seen_timestamp
        {
            return Err(CommandFailure::InvalidTimestamp {
                last_seen_timestamp,
                received_timestamp: timestamp,
            });
        }
        Ok(())
    }

    /// Clean up expired orders
    fn clean_up_expired_orders(&mut self, meta: CommandMeta) {
        self.clean_up_expired_limit_orders(meta.sequence_number, meta.timestamp);
        self.clean_up_expired_pegged_orders(meta.timestamp);
    }

    /// Clean up expired limit orders
    fn clean_up_expired_limit_orders(
        &mut self,
        sequence_number: SequenceNumber,
        timestamp: Timestamp,
    ) {
        while let Some(Reverse((expires_at, order_id))) = self.limit.expiration_queue.peek() {
            if *expires_at > timestamp {
                break;
            }
            let expired = match self.limit.orders.get(order_id) {
                Some(order) => order.is_expired(timestamp),
                None => {
                    self.limit.expiration_queue.pop();
                    continue;
                }
            };

            // Check if the order is actually expired, as the TIF of the order may have changed
            // since the order was added to the expiration queue
            if expired {
                self.remove_limit_order(sequence_number, *order_id);
            }

            self.limit.expiration_queue.pop();
        }
    }

    /// Clean up expired pegged orders
    fn clean_up_expired_pegged_orders(&mut self, timestamp: Timestamp) {
        while let Some(Reverse((expires_at, order_id))) = self.pegged.expiration_queue.peek() {
            if *expires_at > timestamp {
                break;
            }
            let expired = match self.pegged.orders.get(order_id) {
                Some(order) => order.is_expired(timestamp),
                None => {
                    self.pegged.expiration_queue.pop();
                    continue;
                }
            };

            // Check if the order is actually expired, as the TIF of the order may have changed
            // since the order was added to the expiration queue
            if expired {
                self.remove_pegged_order(*order_id);
            }

            self.pegged.expiration_queue.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LimitOrder, MarketOrder, OrderFlags, OrderId, OrderKind, PegReference, PeggedOrder, Price,
        Quantity, QuantityPolicy, Side, TimeInForce, Timestamp,
    };

    #[test]
    fn test_handle_command_meta() {
        let mut book: OrderBook = OrderBook::new("TEST");
        assert!(book.last_sequence_number.is_none());
        assert!(book.last_seen_timestamp.is_none());

        // Expected sequence number is 0
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(1),
            timestamp: Timestamp(0),
        });
        assert_eq!(
            result.unwrap_err(),
            CommandFailure::InvalidSequenceNumber {
                expected_sequence_number: SequenceNumber(0),
                received_sequence_number: SequenceNumber(1),
            }
        );
        assert!(book.last_sequence_number.is_none());
        assert!(book.last_seen_timestamp.is_none());

        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(0),
        });
        assert!(result.is_ok());
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(0)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(0)));

        // Expected sequence number is 1
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(10),
        });
        assert_eq!(
            result.unwrap_err(),
            CommandFailure::InvalidSequenceNumber {
                expected_sequence_number: SequenceNumber(1),
                received_sequence_number: SequenceNumber(0),
            }
        );
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(0)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(0)));

        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(1),
            timestamp: Timestamp(10),
        });
        assert!(result.is_ok());
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(1)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(10)));

        // Timestamp is before the last seen timestamp
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(2),
            timestamp: Timestamp(9),
        });
        assert_eq!(
            result.unwrap_err(),
            CommandFailure::InvalidTimestamp {
                last_seen_timestamp: Timestamp(10),
                received_timestamp: Timestamp(9),
            }
        );
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(1)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(10)));

        // Expected sequence number is 2
        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(3),
            timestamp: Timestamp(10),
        });
        assert_eq!(
            result.unwrap_err(),
            CommandFailure::InvalidSequenceNumber {
                expected_sequence_number: SequenceNumber(2),
                received_sequence_number: SequenceNumber(3),
            }
        );
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(1)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(10)));

        let result = book.handle_command_meta(CommandMeta {
            sequence_number: SequenceNumber(2),
            timestamp: Timestamp(10),
        });
        assert!(result.is_ok());
        assert_eq!(book.last_sequence_number, Some(SequenceNumber(2)));
        assert_eq!(book.last_seen_timestamp, Some(Timestamp(10)));
    }

    #[test]
    fn test_clean_up_expired_limit_orders() {
        let mut book: OrderBook = OrderBook::new("TEST");
        assert_eq!(book.limit.bids.len(), 0);
        assert_eq!(book.limit.orders.len(), 0);
        assert_eq!(book.limit.expiration_queue.len(), 0);

        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );
        assert_eq!(book.limit.bids.len(), 1);
        assert_eq!(book.limit.orders.len(), 1);
        assert_eq!(book.limit.expiration_queue.len(), 1);

        book.add_limit_order(
            SequenceNumber(1),
            OrderId(1),
            LimitOrder::new(
                Price(101),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );
        assert_eq!(book.limit.bids.len(), 2);
        assert_eq!(book.limit.orders.len(), 2);
        assert_eq!(book.limit.expiration_queue.len(), 2);

        book.add_limit_order(
            SequenceNumber(2),
            OrderId(2),
            LimitOrder::new(
                Price(101),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1001))),
            ),
        );
        assert_eq!(book.limit.bids.len(), 2);
        assert_eq!(book.limit.orders.len(), 3);
        assert_eq!(book.limit.expiration_queue.len(), 3);

        book.add_limit_order(
            SequenceNumber(3),
            OrderId(3),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1002))),
            ),
        );
        assert_eq!(book.limit.bids.len(), 2);
        assert_eq!(book.limit.orders.len(), 4);
        assert_eq!(book.limit.expiration_queue.len(), 4);

        // No orders should be expired
        book.clean_up_expired_limit_orders(SequenceNumber(999), Timestamp(999));
        assert_eq!(book.limit.bids.len(), 2);
        assert_eq!(book.limit.orders.len(), 4);
        assert_eq!(book.limit.expiration_queue.len(), 4);

        // Two orders at GTD 1000 should be expired
        book.clean_up_expired_limit_orders(SequenceNumber(1000), Timestamp(1000));
        assert_eq!(book.limit.bids.len(), 2);
        assert_eq!(book.limit.orders.len(), 2);
        assert_eq!(book.limit.expiration_queue.len(), 2);

        // Two remaining orders should be expired
        book.clean_up_expired_limit_orders(SequenceNumber(1002), Timestamp(1002));
        assert_eq!(book.limit.bids.len(), 0);
        assert_eq!(book.limit.orders.len(), 0);
        assert_eq!(book.limit.expiration_queue.len(), 0);
    }

    #[test]
    fn test_clean_up_non_expiring_limit_orders() {
        let mut book: OrderBook = OrderBook::new("TEST");
        assert_eq!(book.limit.bids.len(), 0);
        assert_eq!(book.limit.orders.len(), 0);
        assert_eq!(book.limit.expiration_queue.len(), 0);

        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        assert_eq!(book.limit.bids.len(), 1);
        assert_eq!(book.limit.orders.len(), 1);
        assert_eq!(book.limit.expiration_queue.len(), 0);

        book.clean_up_expired_limit_orders(SequenceNumber(1000), Timestamp(1000));
        assert_eq!(book.limit.bids.len(), 1);
        assert_eq!(book.limit.orders.len(), 1);
        assert_eq!(book.limit.expiration_queue.len(), 0);
    }

    #[test]
    fn test_clean_up_expired_pegged_orders() {
        let mut book: OrderBook = OrderBook::new("TEST");
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 0);
        assert_eq!(book.pegged.expiration_queue.len(), 0);

        book.add_pegged_order(
            SequenceNumber(0),
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 1);
        assert_eq!(book.pegged.expiration_queue.len(), 1);

        book.add_pegged_order(
            SequenceNumber(1),
            OrderId(1),
            PeggedOrder::new(
                PegReference::Market,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1000))),
            ),
        );
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 2);
        assert_eq!(book.pegged.expiration_queue.len(), 2);

        book.add_pegged_order(
            SequenceNumber(2),
            OrderId(2),
            PeggedOrder::new(
                PegReference::MidPrice,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(1001))),
            ),
        );
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 3);
        assert_eq!(book.pegged.expiration_queue.len(), 3);

        book.add_pegged_order(
            SequenceNumber(3),
            OrderId(3),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtd(Timestamp(1002))),
            ),
        );
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 4);
        assert_eq!(book.pegged.expiration_queue.len(), 4);

        // No orders should be expired
        book.clean_up_expired_pegged_orders(Timestamp(999));
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 4);
        assert_eq!(book.pegged.expiration_queue.len(), 4);

        // Two orders at GTD 1000 should be expired
        book.clean_up_expired_pegged_orders(Timestamp(1000));
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 2);
        assert_eq!(book.pegged.expiration_queue.len(), 2);

        // Two remaining orders should be expired
        book.clean_up_expired_pegged_orders(Timestamp(1002));
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 0);
        assert_eq!(book.pegged.expiration_queue.len(), 0);
    }

    /// Context for the execution tests
    struct TestContext {
        next_sequence_number: SequenceNumber,
    }

    impl TestContext {
        /// Create a new test context
        fn new() -> Self {
            Self {
                next_sequence_number: SequenceNumber(0),
            }
        }

        /// Get the command metadata
        fn meta(&mut self) -> CommandMeta {
            let sequence_number = self.next_sequence_number;
            self.next_sequence_number = sequence_number.next();
            CommandMeta {
                sequence_number,
                timestamp: Timestamp(0),
            }
        }
    }

    /// Helper function to get the target order ID from the command outcome
    /// Returns `None` if the command was rejected or the order was cancelled
    fn target_order_id(outcome: &CommandOutcome) -> Option<OrderId> {
        match outcome {
            CommandOutcome::Applied(CommandReport::Submit(command_effects)) => {
                Some(command_effects.target_order().order_id())
            }
            CommandOutcome::Applied(CommandReport::Amend(command_effects)) => {
                Some(command_effects.target_order().order_id())
            }
            _ => None,
        }
    }

    #[test]
    fn test_standard_order_execution() {
        let mut book = OrderBook::new("TEST");
        let mut ctx = TestContext::new();

        // Submit a standard buy order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId::from(meta.sequence_number)),
        )));
        assert_eq!(outcome, expected_outcome);

        let target_order_id = target_order_id(&outcome).unwrap();

        // Amend the buy order
        let outcome = book.execute(&Command {
            meta: ctx.meta(),
            kind: CommandKind::Amend(AmendCmd {
                order_id: target_order_id,
                patch: AmendPatch::Limit(
                    LimitOrderPatch::new()
                        .with_price(Price(101))
                        .with_quantity_policy(QuantityPolicy::Standard {
                            quantity: Quantity(20),
                        }),
                ),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Amend(CommandEffects::new(
            OrderOutcome::new(OrderId(0)),
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit a standard marketable sell order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(101),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        let mut expected_match_result = MatchResult::new(Side::Sell);
        expected_match_result.add_trade(Trade::new(OrderId(0), Price(101), Quantity(10)));

        let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
        expected_order_outcome.set_match_result(expected_match_result);

        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            expected_order_outcome,
        )));
        assert_eq!(outcome, expected_outcome);

        // Cancel the remaining buy order
        let outcome = book.execute(&Command {
            meta: ctx.meta(),
            kind: CommandKind::Cancel(CancelCmd {
                order_id: target_order_id,
                order_kind: OrderKind::Limit,
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Cancel);
        assert_eq!(outcome, expected_outcome);

        // Submit standard buy orders from the best price to the worst price
        for i in 0..10 {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(100 - i),
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(OrderOutcome::new(OrderId::from(meta.sequence_number))),
            ));
            assert_eq!(outcome, expected_outcome);
        }

        // Submit standard sell orders from the best price to the worst price
        for i in 0..10 {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(110 + i),
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(OrderOutcome::new(OrderId::from(meta.sequence_number))),
            ));
            assert_eq!(outcome, expected_outcome);
        }

        // Submit standard marketable buy orders
        let expected_taker_side = Side::Buy;
        let expected_trades = [
            [
                Trade::new(OrderId(14), Price(110), Quantity(100)),
                Trade::new(OrderId(15), Price(111), Quantity(100)),
            ],
            [
                Trade::new(OrderId(16), Price(112), Quantity(100)),
                Trade::new(OrderId(17), Price(113), Quantity(100)),
            ],
            [
                Trade::new(OrderId(18), Price(114), Quantity(100)),
                Trade::new(OrderId(19), Price(115), Quantity(100)),
            ],
            [
                Trade::new(OrderId(20), Price(116), Quantity(100)),
                Trade::new(OrderId(21), Price(117), Quantity(100)),
            ],
            [
                Trade::new(OrderId(22), Price(118), Quantity(100)),
                Trade::new(OrderId(23), Price(119), Quantity(100)),
            ],
        ];
        for trades in expected_trades {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(120),
                        QuantityPolicy::Standard {
                            quantity: Quantity(200),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let mut expected_match_result = MatchResult::new(expected_taker_side);
            for trade in trades {
                expected_match_result.add_trade(trade);
            }

            let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
            expected_order_outcome.set_match_result(expected_match_result);

            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(expected_order_outcome),
            ));
            assert_eq!(outcome, expected_outcome);
        }

        // Submit standard marketable sell orders
        let expected_taker_side = Side::Sell;
        let expected_trades = [
            [
                Trade::new(OrderId(4), Price(100), Quantity(100)),
                Trade::new(OrderId(5), Price(99), Quantity(100)),
            ],
            [
                Trade::new(OrderId(6), Price(98), Quantity(100)),
                Trade::new(OrderId(7), Price(97), Quantity(100)),
            ],
            [
                Trade::new(OrderId(8), Price(96), Quantity(100)),
                Trade::new(OrderId(9), Price(95), Quantity(100)),
            ],
            [
                Trade::new(OrderId(10), Price(94), Quantity(100)),
                Trade::new(OrderId(11), Price(93), Quantity(100)),
            ],
            [
                Trade::new(OrderId(12), Price(92), Quantity(100)),
                Trade::new(OrderId(13), Price(91), Quantity(100)),
            ],
        ];
        for trades in expected_trades {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(90),
                        QuantityPolicy::Standard {
                            quantity: Quantity(200),
                        },
                        OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let mut expected_match_result = MatchResult::new(expected_taker_side);
            for trade in trades {
                expected_match_result.add_trade(trade);
            }

            let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
            expected_order_outcome.set_match_result(expected_match_result);

            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(expected_order_outcome),
            ));
            assert_eq!(outcome, expected_outcome);
        }
    }

    #[test]
    fn test_iceberg_order_execution() {
        let mut book = OrderBook::new("TEST");
        let mut ctx = TestContext::new();

        // Submit an iceberg buy order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(10),
                        hidden_quantity: Quantity(90),
                        replenish_quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId::from(meta.sequence_number)),
        )));
        assert_eq!(outcome, expected_outcome);
        let target_order_id = target_order_id(&outcome).unwrap();

        // Amend the buy order
        let outcome = book.execute(&Command {
            meta: ctx.meta(),
            kind: CommandKind::Amend(AmendCmd {
                order_id: target_order_id,
                patch: AmendPatch::Limit(
                    LimitOrderPatch::new()
                        .with_price(Price(101))
                        .with_quantity_policy(QuantityPolicy::Iceberg {
                            visible_quantity: Quantity(20),
                            hidden_quantity: Quantity(180),
                            replenish_quantity: Quantity(20),
                        }),
                ),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Amend(CommandEffects::new(
            OrderOutcome::new(target_order_id),
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit an iceberg marketable sell order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(101),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(10),
                        hidden_quantity: Quantity(90),
                        replenish_quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        let mut expected_match_result = MatchResult::new(Side::Sell);
        for _ in 0..5 {
            expected_match_result.add_trade(Trade::new(OrderId(0), Price(101), Quantity(20)));
        }

        let mut expected_order_outcome = OrderOutcome::new(OrderId(2));
        expected_order_outcome.set_match_result(expected_match_result);

        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            expected_order_outcome,
        )));
        assert_eq!(outcome, expected_outcome);

        // Cancel the remaining buy order
        let outcome = book.execute(&Command {
            meta: ctx.meta(),
            kind: CommandKind::Cancel(CancelCmd {
                order_id: target_order_id,
                order_kind: OrderKind::Limit,
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Cancel);
        assert_eq!(outcome, expected_outcome);

        // Submit iceberg buy orders
        for _ in 0..10 {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(100),
                        QuantityPolicy::Iceberg {
                            visible_quantity: Quantity(10),
                            hidden_quantity: Quantity(90),
                            replenish_quantity: Quantity(10),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(OrderOutcome::new(OrderId::from(meta.sequence_number))),
            ));
            assert_eq!(outcome, expected_outcome);
        }

        // Submit iceberg sell orders
        for _ in 0..10 {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(110),
                        QuantityPolicy::Iceberg {
                            visible_quantity: Quantity(10),
                            hidden_quantity: Quantity(90),
                            replenish_quantity: Quantity(10),
                        },
                        OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(OrderOutcome::new(OrderId::from(meta.sequence_number))),
            ));
            assert_eq!(outcome, expected_outcome);
        }

        // Submit iceberg marketable buy orders
        let expected_taker_side = Side::Buy;
        let expected_trades = [
            Trade::new(OrderId(14), Price(110), Quantity(10)),
            Trade::new(OrderId(15), Price(110), Quantity(10)),
            Trade::new(OrderId(16), Price(110), Quantity(10)),
            Trade::new(OrderId(17), Price(110), Quantity(10)),
            Trade::new(OrderId(18), Price(110), Quantity(10)),
            Trade::new(OrderId(19), Price(110), Quantity(10)),
            Trade::new(OrderId(20), Price(110), Quantity(10)),
            Trade::new(OrderId(21), Price(110), Quantity(10)),
            Trade::new(OrderId(22), Price(110), Quantity(10)),
            Trade::new(OrderId(23), Price(110), Quantity(10)),
            Trade::new(OrderId(14), Price(110), Quantity(10)),
            Trade::new(OrderId(15), Price(110), Quantity(10)),
            Trade::new(OrderId(16), Price(110), Quantity(10)),
            Trade::new(OrderId(17), Price(110), Quantity(10)),
            Trade::new(OrderId(18), Price(110), Quantity(10)),
            Trade::new(OrderId(19), Price(110), Quantity(10)),
            Trade::new(OrderId(20), Price(110), Quantity(10)),
            Trade::new(OrderId(21), Price(110), Quantity(10)),
            Trade::new(OrderId(22), Price(110), Quantity(10)),
            Trade::new(OrderId(23), Price(110), Quantity(10)),
        ];
        for _ in 0..5 {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(110),
                        QuantityPolicy::Iceberg {
                            visible_quantity: Quantity(20),
                            hidden_quantity: Quantity(180),
                            replenish_quantity: Quantity(20),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let mut expected_match_result = MatchResult::new(expected_taker_side);
            for trade in expected_trades {
                expected_match_result.add_trade(trade);
            }

            let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
            expected_order_outcome.set_match_result(expected_match_result);

            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(expected_order_outcome),
            ));
            assert_eq!(outcome, expected_outcome);
        }

        // Submit iceberg marketable sell orders
        let expected_taker_side = Side::Sell;
        let expected_trades = [
            Trade::new(OrderId(4), Price(100), Quantity(10)),
            Trade::new(OrderId(5), Price(100), Quantity(10)),
            Trade::new(OrderId(6), Price(100), Quantity(10)),
            Trade::new(OrderId(7), Price(100), Quantity(10)),
            Trade::new(OrderId(8), Price(100), Quantity(10)),
            Trade::new(OrderId(9), Price(100), Quantity(10)),
            Trade::new(OrderId(10), Price(100), Quantity(10)),
            Trade::new(OrderId(11), Price(100), Quantity(10)),
            Trade::new(OrderId(12), Price(100), Quantity(10)),
            Trade::new(OrderId(13), Price(100), Quantity(10)),
            Trade::new(OrderId(4), Price(100), Quantity(10)),
            Trade::new(OrderId(5), Price(100), Quantity(10)),
            Trade::new(OrderId(6), Price(100), Quantity(10)),
            Trade::new(OrderId(7), Price(100), Quantity(10)),
            Trade::new(OrderId(8), Price(100), Quantity(10)),
            Trade::new(OrderId(9), Price(100), Quantity(10)),
            Trade::new(OrderId(10), Price(100), Quantity(10)),
            Trade::new(OrderId(11), Price(100), Quantity(10)),
            Trade::new(OrderId(12), Price(100), Quantity(10)),
            Trade::new(OrderId(13), Price(100), Quantity(10)),
        ];
        for _ in 0..5 {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(100),
                        QuantityPolicy::Iceberg {
                            visible_quantity: Quantity(20),
                            hidden_quantity: Quantity(180),
                            replenish_quantity: Quantity(20),
                        },
                        OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let mut expected_match_result = MatchResult::new(expected_taker_side);
            for trade in expected_trades {
                expected_match_result.add_trade(trade);
            }

            let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
            expected_order_outcome.set_match_result(expected_match_result);

            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(expected_order_outcome),
            ));
            assert_eq!(outcome, expected_outcome);
        }
    }

    #[test]
    fn test_market_order_execution() {
        let mut book = OrderBook::new("TEST");
        let mut ctx = TestContext::new();

        // Submit a market buy order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(200), Side::Buy, false)),
            }),
        });
        let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
        expected_order_outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
            requested: Quantity(200),
            available: Quantity(0),
        });

        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            expected_order_outcome,
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit a market sell order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(200), Side::Sell, false)),
            }),
        });
        let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
        expected_order_outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
            requested: Quantity(200),
            available: Quantity(0),
        });

        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            expected_order_outcome,
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit standard buy orders from the best price to the worst price
        for i in 0..10 {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(100 - i),
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(OrderOutcome::new(OrderId::from(meta.sequence_number))),
            ));
            assert_eq!(outcome, expected_outcome);
        }

        // Submit standard sell orders from the best price to the worst price
        for i in 0..10 {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        Price(110 + i),
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                    )),
                }),
            });
            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(OrderOutcome::new(OrderId::from(meta.sequence_number))),
            ));
            assert_eq!(outcome, expected_outcome);
        }

        // Submit market buy orders
        let expected_taker_side = Side::Buy;
        let expected_trades_and_cancel_reasons = [
            (
                vec![
                    Trade::new(OrderId(12), Price(110), Quantity(100)),
                    Trade::new(OrderId(13), Price(111), Quantity(100)),
                    Trade::new(OrderId(14), Price(112), Quantity(10)),
                ],
                None,
            ),
            (
                vec![
                    Trade::new(OrderId(14), Price(112), Quantity(90)),
                    Trade::new(OrderId(15), Price(113), Quantity(100)),
                    Trade::new(OrderId(16), Price(114), Quantity(20)),
                ],
                None,
            ),
            (
                vec![
                    Trade::new(OrderId(16), Price(114), Quantity(80)),
                    Trade::new(OrderId(17), Price(115), Quantity(100)),
                    Trade::new(OrderId(18), Price(116), Quantity(30)),
                ],
                None,
            ),
            (
                vec![
                    Trade::new(OrderId(18), Price(116), Quantity(70)),
                    Trade::new(OrderId(19), Price(117), Quantity(100)),
                    Trade::new(OrderId(20), Price(118), Quantity(40)),
                ],
                None,
            ),
            (
                vec![
                    Trade::new(OrderId(20), Price(118), Quantity(60)),
                    Trade::new(OrderId(21), Price(119), Quantity(100)),
                ],
                Some(CancelReason::InsufficientLiquidity {
                    requested: Quantity(210),
                    available: Quantity(160),
                }),
            ),
        ];
        for (expected_trades, expected_cancel_reason) in expected_trades_and_cancel_reasons {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Market(MarketOrder::new(Quantity(210), Side::Buy, false)),
                }),
            });
            let mut expected_match_result = MatchResult::new(expected_taker_side);
            for trade in expected_trades {
                expected_match_result.add_trade(trade);
            }

            let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
            expected_order_outcome.set_match_result(expected_match_result);
            if let Some(expected_cancel_reason) = expected_cancel_reason {
                expected_order_outcome.set_cancel_reason(expected_cancel_reason);
            }

            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(expected_order_outcome),
            ));
            assert_eq!(outcome, expected_outcome);
        }

        // Submit market sell orders
        let expected_taker_side = Side::Sell;
        let expected_trades_and_cancel_reasons = [
            (
                vec![
                    Trade::new(OrderId(2), Price(100), Quantity(100)),
                    Trade::new(OrderId(3), Price(99), Quantity(100)),
                    Trade::new(OrderId(4), Price(98), Quantity(10)),
                ],
                None,
            ),
            (
                vec![
                    Trade::new(OrderId(4), Price(98), Quantity(90)),
                    Trade::new(OrderId(5), Price(97), Quantity(100)),
                    Trade::new(OrderId(6), Price(96), Quantity(20)),
                ],
                None,
            ),
            (
                vec![
                    Trade::new(OrderId(6), Price(96), Quantity(80)),
                    Trade::new(OrderId(7), Price(95), Quantity(100)),
                    Trade::new(OrderId(8), Price(94), Quantity(30)),
                ],
                None,
            ),
            (
                vec![
                    Trade::new(OrderId(8), Price(94), Quantity(70)),
                    Trade::new(OrderId(9), Price(93), Quantity(100)),
                    Trade::new(OrderId(10), Price(92), Quantity(40)),
                ],
                None,
            ),
            (
                vec![
                    Trade::new(OrderId(10), Price(92), Quantity(60)),
                    Trade::new(OrderId(11), Price(91), Quantity(100)),
                ],
                Some(CancelReason::InsufficientLiquidity {
                    requested: Quantity(210),
                    available: Quantity(160),
                }),
            ),
        ];
        for (expected_trades, expected_cancel_reason) in expected_trades_and_cancel_reasons {
            let meta = ctx.meta();
            let outcome = book.execute(&Command {
                meta,
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Market(MarketOrder::new(Quantity(210), Side::Sell, false)),
                }),
            });
            let mut expected_match_result = MatchResult::new(expected_taker_side);
            for trade in expected_trades {
                expected_match_result.add_trade(trade);
            }

            let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
            expected_order_outcome.set_match_result(expected_match_result);
            if let Some(expected_cancel_reason) = expected_cancel_reason {
                expected_order_outcome.set_cancel_reason(expected_cancel_reason);
            }

            let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(
                CommandEffects::new(expected_order_outcome),
            ));
            assert_eq!(outcome, expected_outcome);
        }
    }

    #[test]
    fn test_pegged_order_execution() {
        let mut book = OrderBook::new("ETH/USD");
        let mut ctx = TestContext::new();

        // Submit a primary pegged buy order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Pegged(PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId::from(meta.sequence_number)),
        )));
        assert_eq!(outcome, expected_outcome);

        let target_order_id = target_order_id(&outcome).unwrap();

        // Amend the buy order to a market pegged buy order
        // The market pegged order will reside at the primary peg level waiting for the new sell order
        let outcome = book.execute(&Command {
            meta: ctx.meta(),
            kind: CommandKind::Amend(AmendCmd {
                order_id: target_order_id,
                patch: AmendPatch::Pegged(
                    PeggedOrderPatch::new()
                        .with_peg_reference(PegReference::Market)
                        .with_quantity(Quantity(200)),
                ),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Amend(CommandEffects::new(
            OrderOutcome::new(target_order_id),
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit a standard sell order
        // The submission command will trigger the market pegged order to be matched with the new sell order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(110),
                    QuantityPolicy::Standard {
                        quantity: Quantity(100),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        let submitted_order_id = OrderId::from(meta.sequence_number);

        let mut expected_triggered_match_result = MatchResult::new(Side::Buy);
        expected_triggered_match_result.add_trade(Trade::new(
            submitted_order_id,
            Price(110),
            Quantity(100),
        ));

        let mut expected_triggered_order_outcome = OrderOutcome::new(target_order_id);
        expected_triggered_order_outcome.set_match_result(expected_triggered_match_result);

        let mut command_effects = CommandEffects::new(OrderOutcome::new(submitted_order_id));
        command_effects.add_triggered_order(expected_triggered_order_outcome);
        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(command_effects));
        assert_eq!(outcome, expected_outcome);

        // Cancel the remaining the market pegged buy order
        let outcome = book.execute(&Command {
            meta: ctx.meta(),
            kind: CommandKind::Cancel(CancelCmd {
                order_id: target_order_id,
                order_kind: OrderKind::Pegged,
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Cancel);
        assert_eq!(outcome, expected_outcome);

        // Submit a mid price pegged buy order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Pegged(PeggedOrder::new(
                    PegReference::MidPrice,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId::from(meta.sequence_number)),
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit a primary pegged buy order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Pegged(PeggedOrder::new(
                    PegReference::Primary,
                    Quantity(100),
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId::from(meta.sequence_number)),
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit a standard buy order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId::from(meta.sequence_number)),
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit a standard sell order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(110),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId::from(meta.sequence_number)),
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit a market pegged sell order - mid price inactive (spread > 1)
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Pegged(PeggedOrder::new(
                    PegReference::Market,
                    Quantity(90),
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        let mut expected_match_result = MatchResult::new(Side::Sell);
        expected_match_result.add_trade(Trade::new(OrderId(6), Price(100), Quantity(10)));
        expected_match_result.add_trade(Trade::new(OrderId(5), Price(100), Quantity(80)));

        let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
        expected_order_outcome.set_match_result(expected_match_result);

        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            expected_order_outcome,
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit a standard buy order
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(109),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            OrderOutcome::new(OrderId::from(meta.sequence_number)),
        )));
        assert_eq!(outcome, expected_outcome);

        // Submit a market pegged sell order - mid price active (spread <= 1)
        let meta = ctx.meta();
        let outcome = book.execute(&Command {
            meta,
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Pegged(PeggedOrder::new(
                    PegReference::Market,
                    Quantity(130),
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        let mut expected_match_result = MatchResult::new(Side::Sell);
        expected_match_result.add_trade(Trade::new(OrderId(9), Price(109), Quantity(10)));
        expected_match_result.add_trade(Trade::new(OrderId(4), Price(109), Quantity(100)));
        expected_match_result.add_trade(Trade::new(OrderId(5), Price(109), Quantity(20)));

        let mut expected_order_outcome = OrderOutcome::new(OrderId::from(meta.sequence_number));
        expected_order_outcome.set_match_result(expected_match_result);

        let expected_outcome = CommandOutcome::Applied(CommandReport::Submit(CommandEffects::new(
            expected_order_outcome,
        )));
        assert_eq!(outcome, expected_outcome);
    }
}
