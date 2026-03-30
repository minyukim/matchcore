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

        let mut cx = self.matching_context(taker_side);

        loop {
            // No more orders in the maker side
            if cx.maker_levels.is_empty() {
                break;
            }

            let Some(queue_entry) = cx.taker_peg_levels[PegReference::Market.as_index()].peek()
            else {
                // No more orders in the taker side
                break;
            };
            let order_id = queue_entry.order_id();

            let Some(order) = cx.pegged_orders.get(&order_id) else {
                // Stale queue entry in the peg level, remove it
                cx.taker_peg_levels[PegReference::Market.as_index()].pop();
                continue;
            };
            if queue_entry.time_priority() != order.time_priority() {
                // Stale queue entry in the peg level, remove it
                cx.taker_peg_levels[PegReference::Market.as_index()].pop();
                continue;
            }
            let (quantity, post_only) = (order.quantity(), order.post_only());

            let mut outcome = OrderOutcome::new(order_id);

            // The post-only order cannot be a taker. Cancel the order.
            if post_only {
                cx.taker_peg_levels[PegReference::Market.as_index()].quantity -= quantity;
                cx.taker_peg_levels[PegReference::Market.as_index()]
                    .remove_head_order(cx.pegged_orders);

                outcome.set_cancel_reason(CancelReason::PostOnlyWouldTake);
                outcomes.push(outcome);
                continue;
            }

            let result = cx.match_order(sequence_number, None, quantity);
            let executed_quantity = result.executed_quantity();
            outcome.set_match_result(result);

            let remaining = quantity - executed_quantity;
            cx.taker_peg_levels[PegReference::Market.as_index()].quantity -= executed_quantity;

            if remaining.is_zero() {
                // The order is fully matched, remove it from the peg level
                cx.taker_peg_levels[PegReference::Market.as_index()]
                    .remove_head_order(cx.pegged_orders);
            } else {
                // The order is partially matched, update the quantity of the order
                cx.pegged_orders
                    .get_mut(&order_id)
                    .unwrap()
                    .update_quantity(remaining);
            }

            outcomes.push(outcome);
        }

        outcomes
    }
}
