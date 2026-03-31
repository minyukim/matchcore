use super::OrderBook;
use crate::{command::*, orders::*, outcome::*, types::*};

impl OrderBook {
    /// Processes the cascading effects after order execution.
    ///
    /// This repeatedly:
    /// 1. Executes all ready price-conditional orders activated by the last-trade price change.
    /// 2. Executes an eligible pegged taker order.
    /// 3. Repeats until no further pegged order can be executed.
    ///
    /// Executing a pegged order may update the last-trade price,
    /// which can activate additional price-conditional orders, forming a cascade.
    pub(super) fn process_order_cascade(
        &mut self,
        meta: CommandMeta,
        was_bid_empty: bool,
        was_ask_empty: bool,
    ) -> Vec<OrderOutcome> {
        let mut outcomes = Vec::new();

        loop {
            while !self.price_conditional.ready_orders.is_empty() {
                let (id, order) = self.price_conditional.ready_orders.pop_front().unwrap();
                let outcome = match order.target_order() {
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
                outcomes.push(outcome);
            }

            let bid_became_non_empty = was_bid_empty && !self.is_side_empty(Side::Buy);
            let ask_became_non_empty = was_ask_empty && !self.is_side_empty(Side::Sell);

            let Some(outcome) = self.match_market_pegged_order(
                meta.sequence_number,
                bid_became_non_empty,
                ask_became_non_empty,
            ) else {
                break;
            };
            outcomes.push(outcome);
        }

        outcomes
    }
}
