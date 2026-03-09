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
        order_id: OrderId,
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
            .get_mut(&order_id)
            .ok_or(RejectReason::OrderNotFound)?;

        let (old_price, old_visible_quantity, old_hidden_quantity, old_expires_at) = (
            order.price(),
            order.visible_quantity(),
            order.hidden_quantity(),
            order.time_in_force().expires_at(),
        );

        patch.apply(order).map_err(RejectReason::CommandError)?;

        // Price change: move the order to the new price level
        if let Some(price) = patch.price
            && price != old_price
        {
            let order = self.remove_limit_order(order_id).unwrap();

            let new_id = OrderId::from(meta.sequence_number);
            let submit_report = self.submit_validated_limit_order(new_id, &order);

            return Ok(AmendReport::from(submit_report).with_new_order_id(new_id));
        }

        if let Some(time_in_force) = patch.time_in_force {
            match time_in_force {
                // An existing order cannot be matched immediately while staying at the same level
                TimeInForce::Ioc | TimeInForce::Fok => {
                    self.remove_limit_order(order_id).unwrap();
                    return Ok(AmendReport::new(
                        OrderProcessingResult::new(order_id).with_cancel_reason(
                            CancelReason::InsufficientLiquidity {
                                available: Quantity(0),
                            },
                        ),
                    ));
                }
                // New expires at
                TimeInForce::Gtd(expires_at)
                    if old_expires_at
                        .is_some_and(|old_expires_at| old_expires_at != expires_at) =>
                {
                    self.limit
                        .expiration_queue
                        .push(Reverse((expires_at, order_id)))
                }
                _ => (),
            }
        }

        let mut report = AmendReport::new(OrderProcessingResult::new(order_id));

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

                let order = self.remove_limit_order(order_id).unwrap();
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
        order_id: OrderId,
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
            .get_mut(&order_id)
            .ok_or(RejectReason::OrderNotFound)?;

        let (old_peg_reference, old_quantity, old_expires_at) = (
            order.peg_reference(),
            order.quantity(),
            order.time_in_force().expires_at(),
        );

        patch.apply(order).map_err(RejectReason::CommandError)?;

        // Peg reference change: move the order to the new peg level
        if let Some(peg_reference) = patch.peg_reference
            && peg_reference != old_peg_reference
        {
            let order = self.remove_pegged_order(order_id).unwrap();

            let new_id = OrderId::from(meta.sequence_number);
            let submit_report = self.submit_validated_pegged_order(new_id, &order);

            return Ok(AmendReport::from(submit_report).with_new_order_id(new_id));
        }

        if let Some(time_in_force) = patch.time_in_force {
            match time_in_force {
                // An existing order cannot be matched immediately while staying at the same level
                TimeInForce::Ioc | TimeInForce::Fok => {
                    self.remove_pegged_order(order_id).unwrap();
                    return Ok(AmendReport::new(
                        OrderProcessingResult::new(order_id).with_cancel_reason(
                            CancelReason::InsufficientLiquidity {
                                available: Quantity(0),
                            },
                        ),
                    ));
                }
                // New expires at
                TimeInForce::Gtd(expires_at)
                    if old_expires_at
                        .is_some_and(|old_expires_at| old_expires_at != expires_at) =>
                {
                    self.pegged
                        .expiration_queue
                        .push(Reverse((expires_at, order_id)))
                }
                _ => (),
            }
        }

        let mut report = AmendReport::new(OrderProcessingResult::new(order_id));

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

                let order = self.remove_pegged_order(order_id).unwrap();
                self.pegged.add_order(new_id, order);

                report = report.with_new_order_id(new_id);
            }
        }

        Ok(report)
    }
}
