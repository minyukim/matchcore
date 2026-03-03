use crate::{LimitOrder, PriceLevel, Side, orderbook::OrderBook};

use std::collections::btree_map::Entry;

impl OrderBook {
    /// Add a limit order to the order book
    pub(super) fn add_limit_order(&mut self, order: LimitOrder) {
        let orders = &mut self.limit_orders;

        let levels = match order.side() {
            Side::Buy => &mut self.limit_bid_levels,
            Side::Sell => &mut self.limit_ask_levels,
        };

        match levels.entry(order.price()) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(orders, order);
            }
            Entry::Vacant(e) => {
                let mut price_level = PriceLevel::new();
                price_level.push(orders, order);
                e.insert(price_level);
            }
        }
    }
}
