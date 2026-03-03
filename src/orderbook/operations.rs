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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LimitOrderSpec, OrderFlags, QuantityPolicy, TimeInForce};

    #[test]
    fn test_add_limit_order() {
        let mut book = OrderBook::new("TEST");
        assert!(book.limit_bid_levels.is_empty());
        assert!(book.limit_ask_levels.is_empty());
        assert!(book.limit_orders.is_empty());

        let order = LimitOrder::new(
            0,
            LimitOrderSpec::new(
                100,
                QuantityPolicy::Standard { quantity: 10 },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(order.clone());
        assert_eq!(book.limit_bid_levels.len(), 1);
        assert!(book.limit_ask_levels.is_empty());
        assert_eq!(book.limit_orders.len(), 1);
        assert_eq!(book.limit_bid_levels.get(&100).unwrap().order_count(), 1);
        assert_eq!(book.limit_orders.get(&0).unwrap(), &order);

        let order = LimitOrder::new(
            1,
            LimitOrderSpec::new(
                100,
                QuantityPolicy::Standard { quantity: 10 },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(order.clone());
        assert_eq!(book.limit_bid_levels.len(), 1);
        assert!(book.limit_ask_levels.is_empty());
        assert_eq!(book.limit_orders.len(), 2);
        assert_eq!(book.limit_bid_levels.get(&100).unwrap().order_count(), 2);
        assert_eq!(book.limit_orders.get(&1).unwrap(), &order);

        let order = LimitOrder::new(
            2,
            LimitOrderSpec::new(
                110,
                QuantityPolicy::Standard { quantity: 10 },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(order.clone());
        assert_eq!(book.limit_bid_levels.len(), 1);
        assert_eq!(book.limit_ask_levels.len(), 1);
        assert_eq!(book.limit_orders.len(), 3);
        assert_eq!(book.limit_bid_levels.get(&100).unwrap().order_count(), 2);
        assert_eq!(book.limit_ask_levels.get(&110).unwrap().order_count(), 1);
        assert_eq!(book.limit_orders.get(&2).unwrap(), &order);

        let order = LimitOrder::new(
            3,
            LimitOrderSpec::new(
                105,
                QuantityPolicy::Standard { quantity: 10 },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(order.clone());
        assert_eq!(book.limit_bid_levels.len(), 1);
        assert_eq!(book.limit_ask_levels.len(), 2);
        assert_eq!(book.limit_orders.len(), 4);
        assert_eq!(book.limit_bid_levels.get(&100).unwrap().order_count(), 2);
        assert_eq!(book.limit_ask_levels.get(&110).unwrap().order_count(), 1);
        assert_eq!(book.limit_ask_levels.get(&105).unwrap().order_count(), 1);
        assert_eq!(book.limit_orders.get(&3).unwrap(), &order);
    }
}
