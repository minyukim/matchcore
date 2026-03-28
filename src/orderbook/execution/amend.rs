use super::OrderBook;
use crate::{OrderId, Quantity, QueueEntry, Side, TimeInForce, command::*, outcome::*};

use std::cmp::Reverse;

impl OrderBook {
    /// Execute an amend command against the order book and return the execution outcome
    pub(super) fn execute_amend(&mut self, meta: CommandMeta, cmd: &AmendCmd) -> CommandOutcome {
        let result = match &cmd.patch {
            AmendPatch::Limit(patch) => self.amend_limit_order(meta, cmd.order_id, patch),
            AmendPatch::Pegged(patch) => self.amend_pegged_order(meta, cmd.order_id, patch),
        };

        match result {
            Ok(effects) => CommandOutcome::Applied(CommandReport::Amend(effects)),
            Err(failure) => CommandOutcome::Rejected(failure),
        }
    }

    /// Amend a limit order
    fn amend_limit_order(
        &mut self,
        meta: CommandMeta,
        id: OrderId,
        patch: &LimitOrderPatch,
    ) -> Result<CommandEffects, CommandFailure> {
        if patch.is_empty() {
            return Err(CommandFailure::InvalidCommand(CommandError::EmptyPatch));
        }

        if patch.has_expired_time_in_force(meta.timestamp) {
            return Err(CommandFailure::InvalidCommand(CommandError::Expired));
        }

        let order = self
            .limit
            .orders
            .get_mut(&id)
            .ok_or(CommandFailure::OrderNotFound)?;

        let (old_price, old_visible_quantity, old_hidden_quantity, old_expires_at) = (
            order.price(),
            order.visible_quantity(),
            order.hidden_quantity(),
            order.time_in_force().expires_at(),
        );

        patch.apply(order).map_err(CommandFailure::InvalidCommand)?;

        // New expires at
        if let Some(TimeInForce::Gtd(expires_at)) = patch.time_in_force
            && old_expires_at.is_none_or(|old| old != expires_at)
        {
            self.limit.expiration_queue.push(Reverse((expires_at, id)));
        }

        let sequence_number = meta.sequence_number;

        // Price change: move the order to the new price level
        if let Some(price) = patch.price
            && price != old_price
        {
            order.update_time_priority(sequence_number);

            let level_id = order.level_id();
            let order = order.clone().into_order();

            self.apply_limit_order_removal(
                sequence_number,
                level_id,
                old_price,
                old_visible_quantity,
                old_hidden_quantity,
                order.side(),
            );

            let mut outcome = OrderOutcome::new(id);

            if !self.has_crossable_order(order.side(), order.price()) {
                if order.is_immediate() {
                    self.limit.orders.remove(&id);

                    outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                        requested: order.total_quantity(),
                        available: Quantity(0),
                    });
                    return Ok(CommandEffects::new(outcome));
                }

                self.apply_limit_order_addition(
                    sequence_number,
                    id,
                    order.price(),
                    order.visible_quantity(),
                    order.hidden_quantity(),
                    order.side(),
                );

                let triggered_orders = self.trigger_market_pegged_orders_as_takers(
                    sequence_number,
                    order.side().opposite(),
                );

                return Ok(CommandEffects::new(outcome).with_triggered_orders(triggered_orders));
            }

            if order.post_only() {
                self.limit.orders.remove(&id);

                outcome.set_cancel_reason(CancelReason::PostOnlyWouldTake);
                return Ok(CommandEffects::new(outcome));
            }

            if order.time_in_force() == TimeInForce::Fok {
                let executable_quantity = self.max_executable_quantity_with_limit_price_unchecked(
                    order.side(),
                    order.price(),
                    order.total_quantity(),
                );
                if executable_quantity < order.total_quantity() {
                    self.limit.orders.remove(&id);

                    outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                        requested: order.total_quantity(),
                        available: executable_quantity,
                    });
                    return Ok(CommandEffects::new(outcome));
                }
            }

            let result =
                self.match_order(sequence_number, order.side(), None, order.total_quantity());
            let executed_quantity = result.executed_quantity();
            outcome.set_match_result(result);

            let remaining_quantity = order.total_quantity() - executed_quantity;
            if remaining_quantity.is_zero() {
                self.limit.orders.remove(&id);

                return Ok(CommandEffects::new(outcome));
            }

            if order.time_in_force() == TimeInForce::Ioc {
                self.limit.orders.remove(&id);

                outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                    requested: order.total_quantity(),
                    available: executed_quantity,
                });
                return Ok(CommandEffects::new(outcome));
            }

            let quantity_policy = order
                .quantity_policy()
                .with_remaining_quantity(remaining_quantity);

            // Update the order in the order book with the new quantity policy
            self.limit
                .orders
                .get_mut(&id)
                .unwrap()
                .update_quantity_policy(quantity_policy);

            self.apply_limit_order_addition(
                sequence_number,
                id,
                order.price(),
                quantity_policy.visible_quantity(),
                quantity_policy.hidden_quantity(),
                order.side(),
            );

            let triggered_orders = self
                .trigger_market_pegged_orders_as_takers(sequence_number, order.side().opposite());

            return Ok(CommandEffects::new(outcome).with_triggered_orders(triggered_orders));
        }

        if let Some(quantity_policy) = patch.quantity_policy
            && (quantity_policy.visible_quantity() != old_visible_quantity
                || quantity_policy.hidden_quantity() != old_hidden_quantity)
        {
            let level_id = order.level_id();
            let level = &mut self.limit.levels[level_id];
            level.visible_quantity =
                level.visible_quantity + quantity_policy.visible_quantity() - old_visible_quantity;
            level.hidden_quantity =
                level.hidden_quantity + quantity_policy.hidden_quantity() - old_hidden_quantity;

            // Lose time priority due to quantity increase
            if quantity_policy.visible_quantity() > old_visible_quantity {
                order.update_time_priority(sequence_number);
                level.push(QueueEntry::new(sequence_number, id));
            }
        }

        Ok(CommandEffects::new(OrderOutcome::new(id)))
    }

    /// Amend a pegged order
    fn amend_pegged_order(
        &mut self,
        meta: CommandMeta,
        id: OrderId,
        patch: &PeggedOrderPatch,
    ) -> Result<CommandEffects, CommandFailure> {
        if patch.is_empty() {
            return Err(CommandFailure::InvalidCommand(CommandError::EmptyPatch));
        }

        if patch.has_expired_time_in_force(meta.timestamp) {
            return Err(CommandFailure::InvalidCommand(CommandError::Expired));
        }

        let order = self
            .pegged
            .orders
            .get_mut(&id)
            .ok_or(CommandFailure::OrderNotFound)?;

        let (old_peg_reference, old_quantity, old_expires_at) = (
            order.peg_reference(),
            order.quantity(),
            order.time_in_force().expires_at(),
        );

        patch.apply(order).map_err(CommandFailure::InvalidCommand)?;

        // New expires at
        if let Some(TimeInForce::Gtd(expires_at)) = patch.time_in_force
            && old_expires_at.is_none_or(|old| old != expires_at)
        {
            self.pegged.expiration_queue.push(Reverse((expires_at, id)));
        }

        let sequence_number = meta.sequence_number;

        // Peg reference change: move the order to the new peg level
        if let Some(peg_reference) = patch.peg_reference
            && peg_reference != old_peg_reference
        {
            order.update_time_priority(sequence_number);

            let order = order.clone().into_order();

            self.pegged
                .apply_order_removal(old_peg_reference, old_quantity, order.side());

            let mut outcome = OrderOutcome::new(id);

            if peg_reference.is_always_maker() {
                self.pegged.apply_order_addition(
                    sequence_number,
                    id,
                    order.peg_reference(),
                    order.quantity(),
                    order.side(),
                );

                return Ok(CommandEffects::new(outcome));
            }

            if self.is_side_empty(order.side().opposite()) {
                if order.is_immediate() {
                    outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                        requested: order.quantity(),
                        available: Quantity(0),
                    });
                    return Ok(CommandEffects::new(outcome));
                }

                self.pegged.apply_order_addition(
                    sequence_number,
                    id,
                    order.peg_reference(),
                    order.quantity(),
                    order.side(),
                );

                return Ok(CommandEffects::new(outcome));
            }

            if order.time_in_force() == TimeInForce::Fok {
                let executable_quantity =
                    self.max_executable_quantity_unchecked(order.side(), order.quantity());
                if executable_quantity < order.quantity() {
                    outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                        requested: order.quantity(),
                        available: executable_quantity,
                    });
                    return Ok(CommandEffects::new(outcome));
                }
            }

            let result = self.match_order(sequence_number, order.side(), None, order.quantity());
            let executed_quantity = result.executed_quantity();
            outcome.set_match_result(result);

            let remaining_quantity = order.quantity() - executed_quantity;
            if remaining_quantity.is_zero() {
                return Ok(CommandEffects::new(outcome));
            }

            if order.time_in_force() == TimeInForce::Ioc {
                outcome.set_cancel_reason(CancelReason::InsufficientLiquidity {
                    requested: order.quantity(),
                    available: executed_quantity,
                });
                return Ok(CommandEffects::new(outcome));
            }

            self.pegged
                .orders
                .get_mut(&id)
                .unwrap()
                .update_quantity(remaining_quantity);

            self.pegged.apply_order_addition(
                sequence_number,
                id,
                order.peg_reference(),
                remaining_quantity,
                order.side(),
            );

            return Ok(CommandEffects::new(outcome));
        }

        if let Some(quantity) = patch.quantity
            && quantity != old_quantity
        {
            let level = match order.side() {
                Side::Buy => &mut self.pegged.bid_levels[order.peg_reference().as_index()],
                Side::Sell => &mut self.pegged.ask_levels[order.peg_reference().as_index()],
            };
            level.quantity = level.quantity + quantity - old_quantity;

            // Lose time priority due to quantity increase
            if quantity > old_quantity {
                order.update_time_priority(sequence_number);
                level.push(QueueEntry::new(sequence_number, id));
            }
        }

        Ok(CommandEffects::new(OrderOutcome::new(id)))
    }
}

#[cfg(test)]
mod tests_amend_limit_order {
    use super::*;
    use crate::{
        AmendCmd, AmendPatch, CommandMeta, CommandOutcome, LimitOrder, LimitOrderPatch, OrderFlags,
        OrderId, Price, Quantity, QuantityPolicy, SequenceNumber, Side, TimeInForce, Timestamp,
    };

    fn amend(
        book: &mut OrderBook,
        seq: u64,
        ts: u64,
        order_id: OrderId,
        patch: LimitOrderPatch,
    ) -> CommandOutcome {
        book.execute_amend(
            CommandMeta {
                sequence_number: SequenceNumber(seq),
                timestamp: Timestamp(ts),
            },
            &AmendCmd {
                order_id,
                patch: AmendPatch::Limit(patch),
            },
        )
    }

    fn unwrap_amend_effects(outcome: CommandOutcome) -> CommandEffects {
        match outcome {
            CommandOutcome::Applied(CommandReport::Amend(effects)) => effects,
            other => panic!("expected applied amend, got: {other:?}"),
        }
    }

    #[test]
    fn reject_empty_patch() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let outcome = amend(&mut book, 1, 0, OrderId(0), LimitOrderPatch::new());

        match outcome {
            CommandOutcome::Rejected(CommandFailure::InvalidCommand(CommandError::EmptyPatch)) => {}
            other => panic!("expected empty patch rejection, got: {other:?}"),
        }
    }

    #[test]
    fn reject_expired_tif() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let outcome = amend(
            &mut book,
            1,
            1000,
            OrderId(0),
            LimitOrderPatch::new()
                .with_price(Price(100))
                .with_time_in_force(TimeInForce::Gtd(Timestamp(1000))),
        );

        match outcome {
            CommandOutcome::Rejected(CommandFailure::InvalidCommand(CommandError::Expired)) => {}
            other => panic!("expected expired rejection, got: {other:?}"),
        }
    }

    #[test]
    fn reject_order_not_found() {
        let mut book = OrderBook::new("TEST");

        let outcome = amend(
            &mut book,
            1,
            0,
            OrderId(999),
            LimitOrderPatch::new().with_price(Price(100)),
        );

        match outcome {
            CommandOutcome::Rejected(CommandFailure::OrderNotFound) => {}
            other => panic!("expected order not found, got: {other:?}"),
        }
    }

    #[test]
    fn price_change_reprioritizes_order_and_move_to_new_price_level() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        let order = book.limit.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(0));
        assert_eq!(order.price(), Price(100));
        assert_eq!(order.total_quantity(), Quantity(10));
        assert_eq!(
            book.limit.get_bid_level(Price(100)).unwrap().queue(),
            &[QueueEntry::new(SequenceNumber(0), OrderId(0))]
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            1,
            0,
            OrderId(0),
            LimitOrderPatch::new()
                .with_price(Price(101))
                .with_quantity_policy(QuantityPolicy::Standard {
                    quantity: Quantity(10),
                }),
        ));
        assert_eq!(effects.target_order().order_id(), OrderId(0));
        assert!(effects.target_order().cancel_reason().is_none());

        let order = book.limit.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(1));
        assert_eq!(order.price(), Price(101));
        assert_eq!(order.total_quantity(), Quantity(10));
        assert_eq!(
            book.limit.get_bid_level(Price(101)).unwrap().queue(),
            &[QueueEntry::new(SequenceNumber(1), OrderId(0))]
        );
        assert!(!book.limit.bids.contains_key(&Price(100)));
    }

    #[test]
    fn price_change_matches_order() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(
            SequenceNumber(1),
            OrderId(1),
            LimitOrder::new(
                Price(101),
                QuantityPolicy::Standard {
                    quantity: Quantity(15),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            2,
            0,
            OrderId(1),
            LimitOrderPatch::new().with_price(Price(100)),
        ));
        assert_eq!(effects.target_order().order_id(), OrderId(1));

        let match_result = effects.target_order().match_result().unwrap();
        assert_eq!(match_result.executed_quantity(), Quantity(10));
        assert_eq!(
            match_result.trades(),
            &[Trade::new(OrderId(0), Price(100), Quantity(10))]
        );

        assert!(!book.limit.orders.contains_key(&OrderId(0)));
        assert!(book.limit.orders.contains_key(&OrderId(1)));

        let order = book.limit.orders.get(&OrderId(1)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(2));
        assert_eq!(order.price(), Price(100));
        assert_eq!(order.total_quantity(), Quantity(5));

        assert!(!book.limit.bids.contains_key(&Price(100)));
        assert!(!book.limit.asks.contains_key(&Price(101)));
        assert_eq!(
            book.limit.get_ask_level(Price(100)).unwrap().queue(),
            &[QueueEntry::new(SequenceNumber(2), OrderId(1))]
        );
    }

    #[test]
    fn gtd_patch_updates_expiration_queue() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            1,
            0,
            OrderId(0),
            LimitOrderPatch::new()
                .with_price(Price(100))
                .with_quantity_policy(QuantityPolicy::Standard {
                    quantity: Quantity(10),
                })
                .with_time_in_force(TimeInForce::Gtd(Timestamp(2000))),
        ));
        assert_eq!(effects.target_order().order_id(), OrderId(0));
        assert!(book.limit.orders.contains_key(&OrderId(0)));
        assert!(!book.limit.expiration_queue.is_empty());
    }

    #[test]
    fn quantity_decrease_no_reprioritization() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(
            SequenceNumber(1),
            OrderId(1),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(20),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        let order = book.limit.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(0));
        assert_eq!(order.price(), Price(100));
        assert_eq!(order.total_quantity(), Quantity(10));

        let level = book.limit.get_bid_level(Price(100)).unwrap();
        assert_eq!(level.visible_quantity, Quantity(30));
        assert_eq!(
            level.queue(),
            &[
                QueueEntry::new(SequenceNumber(0), OrderId(0)),
                QueueEntry::new(SequenceNumber(1), OrderId(1))
            ]
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            2,
            0,
            OrderId(0),
            LimitOrderPatch::new().with_quantity_policy(QuantityPolicy::Standard {
                quantity: Quantity(5),
            }),
        ));
        assert_eq!(effects.target_order().order_id(), OrderId(0));

        let order = book.limit.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(0));
        assert_eq!(order.price(), Price(100));
        assert_eq!(order.total_quantity(), Quantity(5));

        let level = book.limit.get_bid_level(Price(100)).unwrap();
        assert_eq!(level.visible_quantity, Quantity(25));
        assert_eq!(
            level.queue(),
            &[
                QueueEntry::new(SequenceNumber(0), OrderId(0)),
                QueueEntry::new(SequenceNumber(1), OrderId(1))
            ]
        );
    }

    #[test]
    fn quantity_increase_reprioritizes_order() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(
            SequenceNumber(1),
            OrderId(1),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(20),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        let order = book.limit.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(0));
        assert_eq!(order.price(), Price(100));
        assert_eq!(order.total_quantity(), Quantity(10));

        let level = book.limit.get_bid_level(Price(100)).unwrap();
        assert_eq!(level.visible_quantity, Quantity(30));
        assert_eq!(
            level.queue(),
            &[
                QueueEntry::new(SequenceNumber(0), OrderId(0)),
                QueueEntry::new(SequenceNumber(1), OrderId(1))
            ]
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            2,
            0,
            OrderId(0),
            LimitOrderPatch::new().with_quantity_policy(QuantityPolicy::Standard {
                quantity: Quantity(20),
            }),
        ));
        assert_eq!(effects.target_order().order_id(), OrderId(0));

        let order = book.limit.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(2));
        assert_eq!(order.price(), Price(100));
        assert_eq!(order.total_quantity(), Quantity(20));

        let level = book.limit.get_bid_level(Price(100)).unwrap();
        assert_eq!(level.visible_quantity, Quantity(40));
        assert_eq!(
            level.queue(),
            &[
                QueueEntry::new(SequenceNumber(0), OrderId(0)),
                QueueEntry::new(SequenceNumber(1), OrderId(1)),
                QueueEntry::new(SequenceNumber(2), OrderId(0))
            ]
        );
    }
}

#[cfg(test)]
mod tests_amend_pegged_order {
    use super::*;
    use crate::{
        AmendCmd, AmendPatch, CommandMeta, CommandOutcome, LimitOrder, OrderFlags, OrderId,
        PegReference, PeggedOrder, PeggedOrderPatch, Price, Quantity, QuantityPolicy,
        SequenceNumber, Side, TimeInForce, Timestamp, Trade,
    };

    fn amend(
        book: &mut OrderBook,
        seq: u64,
        ts: u64,
        order_id: OrderId,
        patch: PeggedOrderPatch,
    ) -> CommandOutcome {
        book.execute_amend(
            CommandMeta {
                sequence_number: SequenceNumber(seq),
                timestamp: Timestamp(ts),
            },
            &AmendCmd {
                order_id,
                patch: AmendPatch::Pegged(patch),
            },
        )
    }

    fn unwrap_amend_effects(outcome: CommandOutcome) -> CommandEffects {
        match outcome {
            CommandOutcome::Applied(CommandReport::Amend(effects)) => effects,
            other => panic!("expected applied amend, got: {other:?}"),
        }
    }

    #[test]
    fn reject_empty_patch() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            SequenceNumber(0),
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let outcome = amend(&mut book, 1, 0, OrderId(0), PeggedOrderPatch::new());

        match outcome {
            CommandOutcome::Rejected(CommandFailure::InvalidCommand(CommandError::EmptyPatch)) => {}
            other => panic!("expected empty patch rejection, got: {other:?}"),
        }
    }

    #[test]
    fn reject_expired_tif() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            SequenceNumber(0),
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let outcome = amend(
            &mut book,
            1,
            1000,
            OrderId(0),
            PeggedOrderPatch::new()
                .with_quantity(Quantity(10))
                .with_time_in_force(TimeInForce::Gtd(Timestamp(1000))),
        );

        match outcome {
            CommandOutcome::Rejected(CommandFailure::InvalidCommand(CommandError::Expired)) => {}
            other => panic!("expected expired rejection, got: {other:?}"),
        }
    }

    #[test]
    fn reject_order_not_found() {
        let mut book = OrderBook::new("TEST");

        let outcome = amend(
            &mut book,
            1,
            0,
            OrderId(999),
            PeggedOrderPatch::new().with_quantity(Quantity(5)),
        );

        match outcome {
            CommandOutcome::Rejected(CommandFailure::OrderNotFound) => {}
            other => panic!("expected order not found, got: {other:?}"),
        }
    }

    #[test]
    fn peg_reference_change_reprioritizes_order_and_move_to_new_peg_level() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            SequenceNumber(0),
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        let order = book.pegged.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(0));
        assert_eq!(order.peg_reference(), PegReference::Primary);
        assert_eq!(order.quantity(), Quantity(10));
        assert_eq!(
            book.pegged.bid_levels[PegReference::Primary.as_index()].queue(),
            &[QueueEntry::new(SequenceNumber(0), OrderId(0))]
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            1,
            0,
            OrderId(0),
            PeggedOrderPatch::new()
                .with_peg_reference(PegReference::Market)
                .with_quantity(Quantity(10)),
        ));
        assert_eq!(effects.target_order().order_id(), OrderId(0));

        let order = book.pegged.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(1));
        assert_eq!(order.peg_reference(), PegReference::Market);
        assert_eq!(order.quantity(), Quantity(10));
        assert_eq!(
            book.pegged.bid_levels[PegReference::Market.as_index()].queue(),
            &[QueueEntry::new(SequenceNumber(1), OrderId(0))]
        );
    }

    #[test]
    fn peg_reference_change_to_market_matches_order() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(
            SequenceNumber(1),
            OrderId(1),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(15),
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            2,
            0,
            OrderId(1),
            PeggedOrderPatch::new().with_peg_reference(PegReference::Market),
        ));
        assert_eq!(effects.target_order().order_id(), OrderId(1));

        let match_result = effects.target_order().match_result().unwrap();
        assert_eq!(match_result.executed_quantity(), Quantity(10));
        assert_eq!(
            match_result.trades(),
            &[Trade::new(OrderId(0), Price(100), Quantity(10))]
        );

        assert!(!book.limit.orders.contains_key(&OrderId(0)));
        assert!(book.pegged.orders.contains_key(&OrderId(1)));

        let order = book.pegged.orders.get(&OrderId(1)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(2));
        assert_eq!(order.peg_reference(), PegReference::Market);
        assert_eq!(order.quantity(), Quantity(5));
        assert_eq!(
            book.pegged.ask_levels[PegReference::Market.as_index()].queue(),
            &[QueueEntry::new(SequenceNumber(2), OrderId(1))]
        );
    }

    #[test]
    fn gtd_patch_updates_expiration_queue() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            SequenceNumber(0),
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            1,
            0,
            OrderId(0),
            PeggedOrderPatch::new()
                .with_quantity(Quantity(10))
                .with_time_in_force(TimeInForce::Gtd(Timestamp(2000))),
        ));
        assert_eq!(effects.target_order().order_id(), OrderId(0));
        assert!(book.pegged.orders.contains_key(&OrderId(0)));
        assert!(!book.pegged.expiration_queue.is_empty());
    }

    #[test]
    fn quantity_decrease_no_reprioritization() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            SequenceNumber(0),
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(
            SequenceNumber(1),
            OrderId(1),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(20),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        let order = book.pegged.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(0));
        assert_eq!(order.peg_reference(), PegReference::Primary);
        assert_eq!(order.quantity(), Quantity(10));

        let level = &mut book.pegged.bid_levels[PegReference::Primary.as_index()];
        assert_eq!(level.quantity, Quantity(30));
        assert_eq!(
            level.queue(),
            &[
                QueueEntry::new(SequenceNumber(0), OrderId(0)),
                QueueEntry::new(SequenceNumber(1), OrderId(1))
            ]
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            2,
            0,
            OrderId(0),
            PeggedOrderPatch::new().with_quantity(Quantity(5)),
        ));

        assert_eq!(effects.target_order().order_id(), OrderId(0));

        let order = book.pegged.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(0));
        assert_eq!(order.peg_reference(), PegReference::Primary);
        assert_eq!(order.quantity(), Quantity(5));

        let level = &mut book.pegged.bid_levels[PegReference::Primary.as_index()];
        assert_eq!(level.quantity, Quantity(25));
        assert_eq!(
            level.queue(),
            &[
                QueueEntry::new(SequenceNumber(0), OrderId(0)),
                QueueEntry::new(SequenceNumber(1), OrderId(1))
            ]
        );
    }

    #[test]
    fn quantity_increase_reprioritizes_order() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            SequenceNumber(0),
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(
            SequenceNumber(1),
            OrderId(1),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(20),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        let order = book.pegged.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(0));
        assert_eq!(order.peg_reference(), PegReference::Primary);
        assert_eq!(order.quantity(), Quantity(10));

        let level = &mut book.pegged.bid_levels[PegReference::Primary.as_index()];
        assert_eq!(level.quantity, Quantity(30));
        assert_eq!(
            level.queue(),
            &[
                QueueEntry::new(SequenceNumber(0), OrderId(0)),
                QueueEntry::new(SequenceNumber(1), OrderId(1))
            ]
        );

        let effects = unwrap_amend_effects(amend(
            &mut book,
            2,
            0,
            OrderId(0),
            PeggedOrderPatch::new().with_quantity(Quantity(20)),
        ));
        assert_eq!(effects.target_order().order_id(), OrderId(0));

        let order = book.pegged.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.time_priority(), SequenceNumber(2));
        assert_eq!(order.peg_reference(), PegReference::Primary);
        assert_eq!(order.quantity(), Quantity(20));

        let level = &mut book.pegged.bid_levels[PegReference::Primary.as_index()];
        assert_eq!(level.quantity, Quantity(40));
        assert_eq!(
            level.queue(),
            &[
                QueueEntry::new(SequenceNumber(0), OrderId(0)),
                QueueEntry::new(SequenceNumber(1), OrderId(1)),
                QueueEntry::new(SequenceNumber(2), OrderId(0))
            ]
        );
    }
}
