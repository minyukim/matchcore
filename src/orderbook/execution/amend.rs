use super::OrderBook;
use crate::{OrderId, Quantity, Side, TimeInForce, command::*, report::*};

use std::cmp::Reverse;

impl OrderBook {
    /// Execute an amend command against the order book
    /// Returns the execution report for the command
    pub(super) fn execute_amend(&mut self, meta: CommandMeta, cmd: &AmendCmd) -> CommandOutcome {
        let result = match &cmd.patch {
            AmendPatch::Limit(patch) => self.amend_limit_order(meta, cmd.order_id, patch),
            AmendPatch::Pegged(patch) => self.amend_pegged_order(meta, cmd.order_id, patch),
        };

        match result {
            Ok(report) => CommandOutcome::Applied(CommandReport::Amend(report)),
            Err(reason) => CommandOutcome::Rejected(reason),
        }
    }

    /// Amend a limit order
    fn amend_limit_order(
        &mut self,
        meta: CommandMeta,
        id: OrderId,
        patch: &LimitOrderPatch,
    ) -> Result<AmendReport, RejectReason> {
        if patch.is_empty() {
            return Err(RejectReason::CommandError(CommandError::EmptyPatch));
        }

        if patch.has_expired_time_in_force(meta.timestamp) {
            return Err(RejectReason::CommandError(CommandError::Expired));
        }

        let order = self
            .limit
            .orders
            .get_mut(&id)
            .ok_or(RejectReason::OrderNotFound)?;

        let (old_price, old_visible_quantity, old_hidden_quantity, old_expires_at) = (
            order.price(),
            order.visible_quantity(),
            order.hidden_quantity(),
            order.time_in_force().expires_at(),
        );

        // Price change: move the order to the new price level
        if let Some(price) = patch.price
            && price != old_price
        {
            let mut order = self.remove_limit_order(id).unwrap();
            patch
                .apply(&mut order)
                .map_err(RejectReason::CommandError)?;

            let new_id = OrderId::from(meta.sequence_number);
            let submit_report = self.submit_validated_limit_order(new_id, &order);

            return Ok(AmendReport::from(submit_report).with_new_order_id(new_id));
        }

        patch.apply(order).map_err(RejectReason::CommandError)?;

        if let Some(time_in_force) = patch.time_in_force {
            match time_in_force {
                // An existing order cannot be matched immediately while staying at the same level
                TimeInForce::Ioc | TimeInForce::Fok => {
                    self.remove_limit_order(id).unwrap();
                    return Ok(AmendReport::new(
                        OrderProcessingResult::new(id).with_cancel_reason(
                            CancelReason::InsufficientLiquidity {
                                available: Quantity(0),
                            },
                        ),
                    ));
                }
                // New expires at
                TimeInForce::Gtd(expires_at)
                    if old_expires_at.is_none_or(|old_expires_at| old_expires_at != expires_at) =>
                {
                    self.limit.expiration_queue.push(Reverse((expires_at, id)))
                }
                _ => (),
            }
        }

        let mut report = AmendReport::new(OrderProcessingResult::new(id));

        if let Some(quantity_policy) = patch.quantity_policy
            && (quantity_policy.visible_quantity() != old_visible_quantity
                || quantity_policy.hidden_quantity() != old_hidden_quantity)
        {
            let level = match order.side() {
                Side::Buy => self.limit.bid_levels.get_mut(&order.price()).unwrap(),
                Side::Sell => self.limit.ask_levels.get_mut(&order.price()).unwrap(),
            };
            level.visible_quantity =
                level.visible_quantity + quantity_policy.visible_quantity() - old_visible_quantity;
            level.hidden_quantity =
                level.hidden_quantity + quantity_policy.hidden_quantity() - old_hidden_quantity;

            // Lose time-priority due to quantity increase
            if quantity_policy.visible_quantity() > old_visible_quantity {
                let new_id = OrderId::from(meta.sequence_number);
                level.push(new_id);

                let order = self.remove_limit_order(id).unwrap();
                self.limit.add_order(new_id, order);

                report = report.with_new_order_id(new_id);
            }
        }

        Ok(report)
    }

    /// Amend a pegged order
    fn amend_pegged_order(
        &mut self,
        meta: CommandMeta,
        id: OrderId,
        patch: &PeggedOrderPatch,
    ) -> Result<AmendReport, RejectReason> {
        if patch.is_empty() {
            return Err(RejectReason::CommandError(CommandError::EmptyPatch));
        }

        if patch.has_expired_time_in_force(meta.timestamp) {
            return Err(RejectReason::CommandError(CommandError::Expired));
        }

        let order = self
            .pegged
            .orders
            .get_mut(&id)
            .ok_or(RejectReason::OrderNotFound)?;

        let (old_peg_reference, old_quantity, old_expires_at) = (
            order.peg_reference(),
            order.quantity(),
            order.time_in_force().expires_at(),
        );

        // Peg reference change: move the order to the new peg level
        if let Some(peg_reference) = patch.peg_reference
            && peg_reference != old_peg_reference
        {
            let mut order = self.remove_pegged_order(id).unwrap();
            patch
                .apply(&mut order)
                .map_err(RejectReason::CommandError)?;

            let new_id = OrderId::from(meta.sequence_number);
            let submit_report = self.submit_validated_pegged_order(new_id, &order);

            return Ok(AmendReport::from(submit_report).with_new_order_id(new_id));
        }

        patch.apply(order).map_err(RejectReason::CommandError)?;

        if let Some(time_in_force) = patch.time_in_force {
            match time_in_force {
                // An existing order cannot be matched immediately while staying at the same level
                TimeInForce::Ioc | TimeInForce::Fok => {
                    self.remove_pegged_order(id).unwrap();
                    return Ok(AmendReport::new(
                        OrderProcessingResult::new(id).with_cancel_reason(
                            CancelReason::InsufficientLiquidity {
                                available: Quantity(0),
                            },
                        ),
                    ));
                }
                // New expires at
                TimeInForce::Gtd(expires_at)
                    if old_expires_at.is_none_or(|old_expires_at| old_expires_at != expires_at) =>
                {
                    self.pegged.expiration_queue.push(Reverse((expires_at, id)))
                }
                _ => (),
            }
        }

        let mut report = AmendReport::new(OrderProcessingResult::new(id));

        if let Some(quantity) = patch.quantity
            && quantity != old_quantity
        {
            let level = match order.side() {
                Side::Buy => &mut self.pegged.bid_levels[order.peg_reference().as_index()],
                Side::Sell => &mut self.pegged.ask_levels[order.peg_reference().as_index()],
            };
            level.quantity = level.quantity + quantity - old_quantity;

            // Lose time-priority due to quantity increase
            if quantity > old_quantity {
                let new_id = OrderId::from(meta.sequence_number);
                level.push(new_id);

                let order = self.remove_pegged_order(id).unwrap();
                self.pegged.add_order(new_id, order);

                report = report.with_new_order_id(new_id);
            }
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests_amend_limit_order {
    use super::*;
    use crate::{
        AmendCmd, AmendPatch, CommandMeta, CommandOutcome, CommandReport, LimitOrder,
        LimitOrderPatch, OrderFlags, OrderId, Price, Quantity, QuantityPolicy, SequenceNumber,
        Side, TimeInForce, Timestamp,
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

    fn unwrap_amend_report(outcome: CommandOutcome) -> AmendReport {
        match outcome {
            CommandOutcome::Applied(CommandReport::Amend(report)) => report,
            other => panic!("expected applied amend, got: {other:?}"),
        }
    }

    #[test]
    fn reject_empty_patch() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
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
            CommandOutcome::Rejected(RejectReason::CommandError(CommandError::EmptyPatch)) => {}
            other => panic!("expected empty patch rejection, got: {other:?}"),
        }
    }

    #[test]
    fn reject_expired_tif() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
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
            CommandOutcome::Rejected(RejectReason::CommandError(CommandError::Expired)) => {}
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
            CommandOutcome::Rejected(RejectReason::OrderNotFound) => {}
            other => panic!("expected order not found, got: {other:?}"),
        }
    }

    #[test]
    fn price_change_removes_and_resubmits_with_new_id() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_amend_report(amend(
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

        assert_eq!(report.new_order_id(), Some(OrderId(1)));
        assert!(report.amended_order().cancel_reason().is_none());
        assert!(!book.limit.orders.contains_key(&OrderId(0)));

        let new_order = book.limit.orders.get(&OrderId(1)).unwrap();
        assert_eq!(new_order.price(), Price(101));
        assert_eq!(new_order.total_quantity(), Quantity(10));
    }

    #[test]
    fn cancel_ioc_order_on_insufficient_liquidity() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_amend_report(amend(
            &mut book,
            1,
            0,
            OrderId(0),
            LimitOrderPatch::new()
                .with_price(Price(100))
                .with_quantity_policy(QuantityPolicy::Standard {
                    quantity: Quantity(10),
                })
                .with_time_in_force(TimeInForce::Ioc),
        ));

        assert_eq!(report.amended_order().order_id(), OrderId(0));
        assert_eq!(
            report.amended_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                available: Quantity(0)
            })
        );
        assert!(!book.limit.orders.contains_key(&OrderId(0)));
    }

    #[test]
    fn cancel_fok_order_on_insufficient_liquidity() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_amend_report(amend(
            &mut book,
            1,
            0,
            OrderId(0),
            LimitOrderPatch::new()
                .with_price(Price(100))
                .with_quantity_policy(QuantityPolicy::Standard {
                    quantity: Quantity(10),
                })
                .with_time_in_force(TimeInForce::Fok),
        ));

        assert_eq!(
            report.amended_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                available: Quantity(0)
            })
        );
    }

    #[test]
    fn gtd_patch_updates_expiration_queue() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_amend_report(amend(
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

        assert_eq!(report.amended_order().order_id(), OrderId(0));
        assert!(report.new_order_id().is_none());
        assert!(book.limit.orders.contains_key(&OrderId(0)));
        assert!(!book.limit.expiration_queue.is_empty());
    }

    #[test]
    fn quantity_decrease_same_id_level_updated() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
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
            OrderId(1),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(20),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let level = book.limit.bid_levels.get_mut(&Price(100)).unwrap();
        assert_eq!(level.visible_quantity, Quantity(30));
        assert_eq!(level.peek_order_id(&book.limit.orders), Some(OrderId(0)));

        let report = unwrap_amend_report(amend(
            &mut book,
            2,
            0,
            OrderId(0),
            LimitOrderPatch::new().with_quantity_policy(QuantityPolicy::Standard {
                quantity: Quantity(5),
            }),
        ));

        assert_eq!(report.amended_order().order_id(), OrderId(0));
        assert!(report.new_order_id().is_none());

        let order = book.limit.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.total_quantity(), Quantity(5));

        let level = book.limit.bid_levels.get_mut(&Price(100)).unwrap();
        assert_eq!(level.visible_quantity, Quantity(25));
        assert_eq!(level.peek_order_id(&book.limit.orders), Some(OrderId(0)));
    }

    #[test]
    fn quantity_increase_new_id_loses_time_priority() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
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
            OrderId(1),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(20),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let level = book.limit.bid_levels.get_mut(&Price(100)).unwrap();
        assert_eq!(level.visible_quantity, Quantity(30));
        assert_eq!(level.peek_order_id(&book.limit.orders), Some(OrderId(0)));

        let report = unwrap_amend_report(amend(
            &mut book,
            2,
            0,
            OrderId(0),
            LimitOrderPatch::new().with_quantity_policy(QuantityPolicy::Standard {
                quantity: Quantity(20),
            }),
        ));

        assert_eq!(report.new_order_id(), Some(OrderId(2)));
        assert!(!book.limit.orders.contains_key(&OrderId(0)));

        let order = book.limit.orders.get(&OrderId(2)).unwrap();
        assert_eq!(order.total_quantity(), Quantity(20));

        let level = book.limit.bid_levels.get_mut(&Price(100)).unwrap();
        assert_eq!(level.visible_quantity, Quantity(40));
        assert_eq!(level.peek_order_id(&book.limit.orders), Some(OrderId(1)));
    }
}

#[cfg(test)]
mod tests_amend_pegged_order {
    use super::*;
    use crate::{
        AmendCmd, AmendPatch, CommandMeta, CommandOutcome, CommandReport, OrderFlags, OrderId,
        PegReference, PeggedOrder, PeggedOrderPatch, Quantity, SequenceNumber, Side, TimeInForce,
        Timestamp,
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

    fn unwrap_amend_report(outcome: CommandOutcome) -> AmendReport {
        match outcome {
            CommandOutcome::Applied(CommandReport::Amend(report)) => report,
            other => panic!("expected applied amend, got: {other:?}"),
        }
    }

    #[test]
    fn reject_empty_patch() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let outcome = amend(&mut book, 1, 0, OrderId(0), PeggedOrderPatch::new());

        match outcome {
            CommandOutcome::Rejected(RejectReason::CommandError(CommandError::EmptyPatch)) => {}
            other => panic!("expected empty patch rejection, got: {other:?}"),
        }
    }

    #[test]
    fn reject_expired_tif() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
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
            CommandOutcome::Rejected(RejectReason::CommandError(CommandError::Expired)) => {}
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
            CommandOutcome::Rejected(RejectReason::OrderNotFound) => {}
            other => panic!("expected order not found, got: {other:?}"),
        }
    }

    #[test]
    fn peg_reference_change_removes_and_resubmits_with_new_id() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_amend_report(amend(
            &mut book,
            1,
            0,
            OrderId(0),
            PeggedOrderPatch::new()
                .with_peg_reference(PegReference::Market)
                .with_quantity(Quantity(10)),
        ));

        assert_eq!(report.new_order_id(), Some(OrderId(1)));
        assert!(!book.pegged.orders.contains_key(&OrderId(0)));

        let new_order = book.pegged.orders.get(&OrderId(1)).unwrap();
        assert_eq!(new_order.peg_reference(), PegReference::Market);
        assert_eq!(new_order.quantity(), Quantity(10));
    }

    #[test]
    fn cancel_ioc_order_on_insufficient_liquidity() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            OrderId(0),
            PeggedOrder::new(
                PegReference::Market,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_amend_report(amend(
            &mut book,
            1,
            0,
            OrderId(0),
            PeggedOrderPatch::new()
                .with_quantity(Quantity(10))
                .with_time_in_force(TimeInForce::Ioc),
        ));

        assert_eq!(report.amended_order().order_id(), OrderId(0));
        assert_eq!(
            report.amended_order().cancel_reason(),
            Some(&CancelReason::InsufficientLiquidity {
                available: Quantity(0)
            })
        );
        assert!(!book.pegged.orders.contains_key(&OrderId(0)));
    }

    #[test]
    fn gtd_patch_updates_expiration_queue() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );

        let report = unwrap_amend_report(amend(
            &mut book,
            1,
            0,
            OrderId(0),
            PeggedOrderPatch::new()
                .with_quantity(Quantity(10))
                .with_time_in_force(TimeInForce::Gtd(Timestamp(2000))),
        ));

        assert_eq!(report.amended_order().order_id(), OrderId(0));
        assert!(report.new_order_id().is_none());
        assert!(book.pegged.orders.contains_key(&OrderId(0)));
        assert!(!book.pegged.expiration_queue.is_empty());
    }

    #[test]
    fn quantity_decrease_same_id_level_updated() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(
            OrderId(1),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(20),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        let level = &mut book.pegged.bid_levels[PegReference::Primary.as_index()];
        assert_eq!(level.quantity, Quantity(30));
        assert_eq!(level.peek_order_id(&book.pegged.orders), Some(OrderId(0)));

        let report = unwrap_amend_report(amend(
            &mut book,
            2,
            0,
            OrderId(0),
            PeggedOrderPatch::new().with_quantity(Quantity(5)),
        ));

        assert_eq!(report.amended_order().order_id(), OrderId(0));
        assert!(report.new_order_id().is_none());

        let order = book.pegged.orders.get(&OrderId(0)).unwrap();
        assert_eq!(order.quantity(), Quantity(5));

        let level = &mut book.pegged.bid_levels[PegReference::Primary.as_index()];
        assert_eq!(level.quantity, Quantity(25));
        assert_eq!(level.peek_order_id(&book.pegged.orders), Some(OrderId(0)));
    }

    #[test]
    fn quantity_increase_new_id_loses_time_priority() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(
            OrderId(1),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(20),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        let level = &mut book.pegged.bid_levels[PegReference::Primary.as_index()];
        assert_eq!(level.quantity, Quantity(30));
        assert_eq!(level.peek_order_id(&book.pegged.orders), Some(OrderId(0)));

        let report = unwrap_amend_report(amend(
            &mut book,
            2,
            0,
            OrderId(0),
            PeggedOrderPatch::new().with_quantity(Quantity(20)),
        ));

        assert_eq!(report.new_order_id(), Some(OrderId(2)));
        assert!(!book.pegged.orders.contains_key(&OrderId(0)));

        let order = book.pegged.orders.get(&OrderId(2)).unwrap();
        assert_eq!(order.quantity(), Quantity(20));

        let level = &mut book.pegged.bid_levels[PegReference::Primary.as_index()];
        assert_eq!(level.quantity, Quantity(40));
        assert_eq!(level.peek_order_id(&book.pegged.orders), Some(OrderId(1)));
    }
}
