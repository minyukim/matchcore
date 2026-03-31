use super::OrderBook;
use crate::{command::*, orders::*, outcome::*, types::*};

impl OrderBook {
    /// Process triggered orders at the given conditions
    pub(super) fn process_triggered_orders(
        &mut self,
        meta: CommandMeta,
        mut prev_trade_price: Option<Price>,
        was_bid_empty: bool,
        was_ask_empty: bool,
    ) -> Vec<OrderOutcome> {
        let mut outcomes = Vec::new();

        loop {
            let curr_trade_price = self.last_trade_price;
            match (prev_trade_price, curr_trade_price) {
                (Some(prev), Some(curr)) => {
                    let orders = self.price_conditional.drain_levels(prev, curr);
                    self.apply_activated_price_conditional_orders(meta, orders, &mut outcomes);
                }
                (None, Some(curr)) => {
                    let orders = self.price_conditional.drain_pre_trade_level_at_price(curr);
                    self.apply_activated_price_conditional_orders(meta, orders, &mut outcomes);
                }
                _ => {}
            }
            prev_trade_price = curr_trade_price;

            let bid_became_non_empty = was_bid_empty && !self.is_side_empty(Side::Buy);
            let ask_became_non_empty = was_ask_empty && !self.is_side_empty(Side::Sell);

            let Some(order_outcome) = self.match_market_pegged_order(
                meta.sequence_number,
                bid_became_non_empty,
                ask_became_non_empty,
            ) else {
                break;
            };
            outcomes.push(order_outcome);
        }

        outcomes
    }

    /// Apply the activated price-conditional orders to the order book and add the resulting order outcomes
    fn apply_activated_price_conditional_orders(
        &mut self,
        meta: CommandMeta,
        orders: Vec<(OrderId, PriceConditionalOrder)>,
        outcomes: &mut Vec<OrderOutcome>,
    ) {
        for (id, order) in orders {
            let order_outcome = match order.target_order() {
                TriggerOrder::Market(order) => {
                    self.submit_validated_market_order(meta.sequence_number, id, order)
                }
                TriggerOrder::Limit(order) => {
                    if order.is_expired(meta.timestamp) {
                        continue;
                    }
                    self.submit_validated_limit_order(meta.sequence_number, id, order)
                }
            };
            outcomes.push(order_outcome);
        }
    }
}
