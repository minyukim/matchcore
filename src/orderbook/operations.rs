use super::{LimitBook, OrderBook, PeggedBook};
use crate::{LimitOrder, PeggedOrder, PriceLevel, Side};

use std::{cmp::Reverse, collections::btree_map::Entry};

impl OrderBook {
    /// Add a limit order to the order book
    pub(super) fn add_limit_order(&mut self, order: LimitOrder) {
        self.limit.add_order(order);
    }

    /// Add a pegged order to the order book
    #[allow(unused)]
    pub(super) fn add_pegged_order(&mut self, order: PeggedOrder) {
        self.pegged.add_order(order);
    }
}

impl LimitBook {
    /// Add a limit order to the order book
    pub(super) fn add_order(&mut self, order: LimitOrder) {
        if let Some(expires_at) = order.expires_at() {
            self.expiration_queue
                .push(Reverse((expires_at, order.id())));
        }

        let orders = &mut self.orders;

        let levels = match order.side() {
            Side::Buy => &mut self.bid_levels,
            Side::Sell => &mut self.ask_levels,
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

impl PeggedBook {
    /// Add a pegged order to the order book
    pub(super) fn add_order(&mut self, order: PeggedOrder) {
        if let Some(expires_at) = order.expires_at() {
            self.expiration_queue
                .push(Reverse((expires_at, order.id())));
        }

        let orders = &mut self.orders;

        let levels = match order.side() {
            Side::Buy => &mut self.bid_levels,
            Side::Sell => &mut self.ask_levels,
        };
        levels[order.peg_reference().as_index()].push_order(orders, order);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LimitOrderSpec, OrderBook, OrderFlags, OrderId, PegReference, PeggedOrderSpec, Price,
        Quantity, QuantityPolicy, TimeInForce,
    };

    #[test]
    fn test_add_limit_order() {
        let mut book = OrderBook::new("TEST");
        assert!(book.limit.bid_levels.is_empty());
        assert!(book.limit.ask_levels.is_empty());
        assert!(book.limit.orders.is_empty());

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
        assert_eq!(book.limit.bid_levels.len(), 1);
        assert!(book.limit.ask_levels.is_empty());
        assert_eq!(book.limit.orders.len(), 1);
        assert_eq!(
            book.limit
                .bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            1
        );
        assert_eq!(book.limit.orders.get(&OrderId(0)).unwrap(), &order);

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
        assert_eq!(book.limit.bid_levels.len(), 1);
        assert!(book.limit.ask_levels.is_empty());
        assert_eq!(book.limit.orders.len(), 2);
        assert_eq!(
            book.limit
                .bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            2
        );
        assert_eq!(book.limit.orders.get(&OrderId(1)).unwrap(), &order);

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
        assert_eq!(book.limit.bid_levels.len(), 1);
        assert_eq!(book.limit.ask_levels.len(), 1);
        assert_eq!(book.limit.orders.len(), 3);
        assert_eq!(
            book.limit
                .bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            2
        );
        assert_eq!(
            book.limit
                .ask_levels
                .get(&Price(110))
                .unwrap()
                .order_count(),
            1
        );
        assert_eq!(book.limit.orders.get(&OrderId(2)).unwrap(), &order);

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
        assert_eq!(book.limit.bid_levels.len(), 1);
        assert_eq!(book.limit.ask_levels.len(), 2);
        assert_eq!(book.limit.orders.len(), 4);
        assert_eq!(
            book.limit
                .bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            2
        );
        assert_eq!(
            book.limit
                .ask_levels
                .get(&Price(110))
                .unwrap()
                .order_count(),
            1
        );
        assert_eq!(
            book.limit
                .ask_levels
                .get(&Price(105))
                .unwrap()
                .order_count(),
            1
        );
        assert_eq!(book.limit.orders.get(&OrderId(3)).unwrap(), &order);
    }

    #[test]
    fn test_add_pegged_order() {
        let mut book = OrderBook::new("TEST");
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert!(book.pegged.orders.is_empty());

        let order = PeggedOrder::new(
            OrderId(0),
            PeggedOrderSpec::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(order.clone());
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 1);
        assert_eq!(book.pegged.orders.get(&OrderId(0)).unwrap(), &order);

        let order = PeggedOrder::new(
            OrderId(1),
            PeggedOrderSpec::new(
                PegReference::Market,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(order.clone());
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 2);
        assert_eq!(book.pegged.orders.get(&OrderId(1)).unwrap(), &order);

        let order = PeggedOrder::new(
            OrderId(2),
            PeggedOrderSpec::new(
                PegReference::MidPrice,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(order.clone());
        for (peg, count) in [
            (PegReference::Primary, 1),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 3);
        assert_eq!(book.pegged.orders.get(&OrderId(2)).unwrap(), &order);

        let order = PeggedOrder::new(
            OrderId(3),
            PeggedOrderSpec::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(order.clone());
        for (peg, count) in [
            (PegReference::Primary, 2),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 0),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 4);
        assert_eq!(book.pegged.orders.get(&OrderId(3)).unwrap(), &order);

        let order = PeggedOrder::new(
            OrderId(4),
            PeggedOrderSpec::new(
                PegReference::MidPrice,
                Quantity(10),
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(order.clone());
        for (peg, count) in [
            (PegReference::Primary, 2),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 5);
        assert_eq!(book.pegged.orders.get(&OrderId(4)).unwrap(), &order);

        let order = PeggedOrder::new(
            OrderId(5),
            PeggedOrderSpec::new(
                PegReference::MidPrice,
                Quantity(10),
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(order.clone());
        for (peg, count) in [
            (PegReference::Primary, 2),
            (PegReference::Market, 1),
            (PegReference::MidPrice, 1),
        ] {
            assert_eq!(book.pegged.bid_levels[peg.as_index()].order_count(), count);
        }
        for (peg, count) in [
            (PegReference::Primary, 0),
            (PegReference::Market, 0),
            (PegReference::MidPrice, 2),
        ] {
            assert_eq!(book.pegged.ask_levels[peg.as_index()].order_count(), count);
        }
        assert_eq!(book.pegged.orders.len(), 6);
        assert_eq!(book.pegged.orders.get(&OrderId(5)).unwrap(), &order);
    }
}
