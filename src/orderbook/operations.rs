use super::OrderBook;
use crate::{LimitOrder, PriceLevel, Side};

use std::{cmp::Reverse, collections::btree_map::Entry};

impl OrderBook {
    /// Add a limit order to the order book
    pub(super) fn add_limit_order(&mut self, order: LimitOrder) {
        if let Some(expires_at) = order.expires_at() {
            self.limit_expiration_queue
                .push(Reverse((expires_at, order.id())));
        }

        let orders = &mut self.limit_orders;

        let levels = match order.side() {
            Side::Buy => &mut self.limit_bid_levels,
            Side::Sell => &mut self.limit_ask_levels,
        };

        match levels.entry(order.price()) {
            Entry::Occupied(mut e) => {
                e.get_mut().push_order(orders, order);
            }
            Entry::Vacant(e) => {
                let mut price_level = PriceLevel::new();
                price_level.push_order(orders, order);
                e.insert(price_level);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LimitOrderSpec, OrderFlags, OrderId, Price, Quantity, QuantityPolicy, TimeInForce,
    };

    #[test]
    fn test_add_limit_order() {
        let mut book = OrderBook::new("TEST");
        assert!(book.limit_bid_levels.is_empty());
        assert!(book.limit_ask_levels.is_empty());
        assert!(book.limit_orders.is_empty());

        let order = LimitOrder::new(
            OrderId(0),
            LimitOrderSpec::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(order.clone());
        assert_eq!(book.limit_bid_levels.len(), 1);
        assert!(book.limit_ask_levels.is_empty());
        assert_eq!(book.limit_orders.len(), 1);
        assert_eq!(
            book.limit_bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            1
        );
        assert_eq!(book.limit_orders.get(&OrderId(0)).unwrap(), &order);

        let order = LimitOrder::new(
            OrderId(1),
            LimitOrderSpec::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(order.clone());
        assert_eq!(book.limit_bid_levels.len(), 1);
        assert!(book.limit_ask_levels.is_empty());
        assert_eq!(book.limit_orders.len(), 2);
        assert_eq!(
            book.limit_bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            2
        );
        assert_eq!(book.limit_orders.get(&OrderId(1)).unwrap(), &order);

        let order = LimitOrder::new(
            OrderId(2),
            LimitOrderSpec::new(
                Price(110),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(order.clone());
        assert_eq!(book.limit_bid_levels.len(), 1);
        assert_eq!(book.limit_ask_levels.len(), 1);
        assert_eq!(book.limit_orders.len(), 3);
        assert_eq!(
            book.limit_bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            2
        );
        assert_eq!(
            book.limit_ask_levels
                .get(&Price(110))
                .unwrap()
                .order_count(),
            1
        );
        assert_eq!(book.limit_orders.get(&OrderId(2)).unwrap(), &order);

        let order = LimitOrder::new(
            OrderId(3),
            LimitOrderSpec::new(
                Price(105),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(order.clone());
        assert_eq!(book.limit_bid_levels.len(), 1);
        assert_eq!(book.limit_ask_levels.len(), 2);
        assert_eq!(book.limit_orders.len(), 4);
        assert_eq!(
            book.limit_bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            2
        );
        assert_eq!(
            book.limit_ask_levels
                .get(&Price(110))
                .unwrap()
                .order_count(),
            1
        );
        assert_eq!(
            book.limit_ask_levels
                .get(&Price(105))
                .unwrap()
                .order_count(),
            1
        );
        assert_eq!(book.limit_orders.get(&OrderId(3)).unwrap(), &order);
    }
}
