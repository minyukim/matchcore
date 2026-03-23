use super::matching::match_order;
use crate::{CancelReason, OrderBook, OrderOutcome, SequenceNumber, types::*};

impl OrderBook {
    /// Trigger the opposite side of the conditional orders to become active takers.
    ///
    /// It iterates over the active peg levels of the taker side, and matches the orders against
    /// the orders at the maker side. It stops when any one side is exhausted.
    ///
    /// Returns a vector of `OrderOutcome` structs containing the outcomes of the order execution.
    pub(crate) fn trigger_opposite_side_takers(
        &mut self,
        sequence_number: SequenceNumber,
        taker_side: Side,
    ) -> Vec<OrderOutcome> {
        let mut outcomes = Vec::new();

        let (
            taker_side_best_price,
            maker_side_price_to_level_id,
            price_levels,
            maker_side_peg_levels,
            taker_side_peg_levels,
        ) = match taker_side {
            Side::Buy => (
                self.best_bid_price(),
                &mut self.limit.asks,
                &mut self.limit.levels,
                &mut self.pegged.ask_levels,
                &mut self.pegged.bid_levels,
            ),
            Side::Sell => (
                self.best_ask_price(),
                &mut self.limit.bids,
                &mut self.limit.levels,
                &mut self.pegged.bid_levels,
                &mut self.pegged.ask_levels,
            ),
        };

        let pegged_orders = &mut self.pegged.orders;

        let active_peg_level = &mut taker_side_peg_levels[PegReference::Market.as_index()];

        loop {
            // No more orders in the maker side
            if maker_side_price_to_level_id.is_empty() {
                break;
            }

            let Some(queue_entry) = active_peg_level.peek() else {
                // No more orders in the taker side
                break;
            };
            let order_id = queue_entry.order_id();

            let Some(order) = pegged_orders.get(&order_id) else {
                // Stale queue entry in the peg level, remove it
                active_peg_level.pop();
                continue;
            };
            if queue_entry.time_priority() != order.time_priority() {
                // Stale queue entry in the peg level, remove it
                active_peg_level.pop();
                continue;
            }
            let (quantity, post_only) = (order.quantity(), order.post_only());

            let mut outcome = OrderOutcome::new(order_id);

            // The post-only order cannot be a taker. Cancel the order.
            if post_only {
                active_peg_level.quantity -= quantity;
                active_peg_level.remove_head_order(pegged_orders);

                outcome.set_cancel_reason(CancelReason::PostOnlyWouldTake);
                outcomes.push(outcome);
                continue;
            }

            let result = match_order(
                sequence_number,
                taker_side,
                taker_side_best_price,
                maker_side_price_to_level_id,
                price_levels,
                &mut self.limit.orders,
                maker_side_peg_levels,
                pegged_orders,
                None,
                quantity,
            );
            self.last_trade_price = result.last_trade_price();
            let executed_quantity = result.executed_quantity();
            outcome.set_match_result(result);

            let remaining = quantity - executed_quantity;
            active_peg_level.quantity -= executed_quantity;

            if remaining.is_zero() {
                // The order is fully matched, remove it from the peg level
                active_peg_level.remove_head_order(pegged_orders);
            } else {
                // The order is partially matched, update the quantity of the order
                pegged_orders
                    .get_mut(&order_id)
                    .unwrap()
                    .update_quantity(remaining);
            }

            outcomes.push(outcome);
        }

        outcomes
    }
}
