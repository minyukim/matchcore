use super::OrderBook;
use crate::{command::*, orders::*, outcome::*};

impl OrderBook {
    /// Apply the activated price-conditional orders to the order book
    /// and add the resulting order outcomes to the command effects
    pub(super) fn apply_activated_price_conditional_orders(
        &mut self,
        meta: CommandMeta,
        orders: Vec<(OrderId, PriceConditionalOrder)>,
        effects: &mut CommandEffects,
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
            effects.add_triggered_order(order_outcome);
        }
    }
}
