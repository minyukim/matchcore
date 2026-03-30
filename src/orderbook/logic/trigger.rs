use crate::{OrderBook, OrderOutcome, PegReference, SequenceNumber, Side};

impl OrderBook {
    /// Trigger the market pegged orders as takers.
    ///
    /// It iterates over the market pegged level of the taker side, and matches the orders against
    /// the orders at the maker side. It stops when any one side is exhausted.
    ///
    /// Returns a vector of `OrderOutcome` structs containing the outcomes of the order execution.
    pub(crate) fn trigger_market_pegged_orders_as_takers(
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

            let outcome =
                cx.match_taker_market_pegged_order(sequence_number, order_id, quantity, post_only);

            outcomes.push(outcome);
        }

        outcomes
    }
}
