use super::matching::match_order;
use crate::{CancelReason, OrderBook, OrderOutcome, types::*};

impl OrderBook {
    /// Trigger the opposite side of the conditional orders to become active takers.
    ///
    /// It iterates over the active peg levels of the taker side, and matches the orders against
    /// the orders at the maker side. It stops when any one side is exhausted.
    ///
    /// Returns a vector of `OrderOutcome` structs containing the outcomes of the order execution.
    pub(crate) fn trigger_opposite_side_takers(&mut self, taker_side: Side) -> Vec<OrderOutcome> {
        let mut outcomes = Vec::new();

        let (
            taker_side_best_price,
            maker_side_price_levels,
            maker_side_peg_levels,
            taker_side_peg_levels,
        ) = match taker_side {
            Side::Buy => (
                self.best_bid_price(),
                &mut self.limit.ask_levels,
                &mut self.pegged.ask_levels,
                &mut self.pegged.bid_levels,
            ),
            Side::Sell => (
                self.best_ask_price(),
                &mut self.limit.bid_levels,
                &mut self.pegged.bid_levels,
                &mut self.pegged.ask_levels,
            ),
        };

        let pegged_orders = &mut self.pegged.orders;

        let active_peg_level = &mut taker_side_peg_levels[PegReference::Market.as_index()];

        loop {
            // No more orders in the maker side
            if maker_side_price_levels.is_empty() {
                break;
            }

            let order_id = active_peg_level.peek_order_id(pegged_orders);

            // No more orders in the taker side
            let Some(order_id) = order_id else {
                break;
            };

            let (quantity, post_only) = {
                // The order is guaranteed to exist because the order ID is found in the peg level
                let order = pegged_orders.get(&order_id).unwrap();
                (order.quantity(), order.post_only())
            };

            // The post-only order cannot be a taker. Cancel the order.
            if post_only {
                outcomes.push(
                    OrderOutcome::new(order_id).with_cancel_reason(CancelReason::PostOnlyWouldTake),
                );
                continue;
            }

            let result = match_order(
                taker_side,
                taker_side_best_price,
                maker_side_price_levels,
                &mut self.limit.orders,
                maker_side_peg_levels,
                pegged_orders,
                None,
                quantity,
            );
            self.last_trade_price = result.last_trade_price();

            let remaining = quantity - result.executed_quantity();
            if remaining.is_zero() {
                // The order is fully matched, remove it from the peg level
                active_peg_level.remove_head_order(pegged_orders);
            } else {
                // The order is partially matched, update the quantity
                pegged_orders
                    .get_mut(&order_id)
                    .unwrap()
                    .update_quantity(remaining);
            }

            outcomes.push(OrderOutcome::new(order_id).with_match_result(result));
        }

        outcomes
    }
}
