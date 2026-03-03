use crate::{
    CancelReason, OrderProcessingResult, Timestamp,
    orderbook::{OrderBook, matching::match_order},
    types::*,
};

impl OrderBook {
    /// Trigger the opposite side of the conditional orders to become active takers.
    ///
    /// It iterates over the active peg levels of the taker side, and matches the orders against
    /// the orders in the maker side. It stops when any one side is exhausted.
    ///
    /// Returns a vector of `OrderProcessingResult` structs containing the results of the matching.
    pub(super) fn trigger_opposite_side_takers(
        &mut self,
        taker_side: Side,
        timestamp: Timestamp,
    ) -> Vec<OrderProcessingResult> {
        let mut results = Vec::new();

        let (
            taker_side_best_price,
            maker_side_price_levels,
            maker_side_peg_levels,
            taker_side_peg_levels,
        ) = match taker_side {
            Side::Buy => (
                self.best_bid(),
                &mut self.limit_ask_levels,
                &mut self.peg_ask_levels,
                &mut self.peg_bid_levels,
            ),
            Side::Sell => (
                self.best_ask(),
                &mut self.limit_bid_levels,
                &mut self.peg_bid_levels,
                &mut self.peg_ask_levels,
            ),
        };

        let pegged_orders = &mut self.pegged_orders;

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

            let (quantity, expired, post_only) = {
                // The order is guaranteed to exist because the order ID is found in the peg level
                let order = pegged_orders.get(&order_id).unwrap();
                (
                    order.quantity(),
                    order.is_expired(timestamp),
                    order.post_only(),
                )
            };

            // The order is expired, remove it from the peg level
            if expired {
                active_peg_level.remove_head_order(pegged_orders);
                continue;
            }

            // The post-only order cannot be a taker. Cancel the order.
            if post_only {
                results.push(
                    OrderProcessingResult::new(order_id)
                        .with_cancel_reason(CancelReason::PostOnlyWouldTake),
                );
                continue;
            }

            let result = match_order(
                taker_side,
                taker_side_best_price,
                maker_side_price_levels,
                &mut self.limit_orders,
                maker_side_peg_levels,
                pegged_orders,
                None,
                quantity,
                timestamp,
            );
            self.last_trade_price = result.last_trade_price();

            let remaining = quantity - result.executed_quantity();
            if remaining == 0 {
                // The order is fully matched, remove it from the peg level
                active_peg_level.remove_head_order(pegged_orders);
            } else {
                // The order is partially matched, update the quantity
                pegged_orders
                    .get_mut(&order_id)
                    .unwrap()
                    .update_quantity(remaining);
            }

            results.push(OrderProcessingResult::new(order_id).with_match_result(result));
        }

        results
    }
}
