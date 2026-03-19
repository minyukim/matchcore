use crate::{
    LimitBook, LimitOrder, OrderBook, OrderId, PegReference, PeggedBook, PeggedOrder, PriceLevel,
    QueueEntry, RestingLimitOrder, RestingPeggedOrder, SequenceNumber, Side,
};

use std::{cmp::Reverse, collections::btree_map::Entry};

impl OrderBook {
    /// Add a limit order to the order book
    pub(crate) fn add_limit_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: LimitOrder,
    ) {
        self.limit.add_order(sequence_number, id, order);
    }

    /// Remove a limit order from the order book
    pub(crate) fn remove_limit_order(
        &mut self,
        sequence_number: SequenceNumber,
        order_id: OrderId,
    ) -> Option<LimitOrder> {
        let order = self.limit.orders.remove(&order_id)?.into_order();

        let levels = match order.side() {
            Side::Buy => &mut self.limit.bid_levels,
            Side::Sell => &mut self.limit.ask_levels,
        };
        let level = levels.get_mut(&order.price()).unwrap();

        level.on_order_removed(order.visible_quantity(), order.hidden_quantity());
        if level.is_empty() {
            let (best_price, peg_levels) = match order.side() {
                Side::Buy => (
                    levels.keys().next_back().copied().unwrap(),
                    &mut self.pegged.bid_levels,
                ),
                Side::Sell => (
                    levels.keys().next().copied().unwrap(),
                    &mut self.pegged.ask_levels,
                ),
            };
            if order.price() == best_price {
                // Only the same side primary peg reprice matters on the best price level removal
                peg_levels[PegReference::Primary.as_index()].repriced_at = sequence_number;
            }

            levels.remove(&order.price());
        }

        Some(order)
    }

    /// Add a pegged order to the order book
    pub(crate) fn add_pegged_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: PeggedOrder,
    ) {
        self.pegged.add_order(sequence_number, id, order);
    }

    /// Remove a pegged order from the order book
    pub(crate) fn remove_pegged_order(&mut self, order_id: OrderId) -> Option<PeggedOrder> {
        self.pegged.remove_order(order_id)
    }
}

impl LimitBook {
    /// Add a limit order to the order book
    pub(crate) fn add_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: LimitOrder,
    ) {
        if let Some(expires_at) = order.expires_at() {
            self.expiration_queue.push(Reverse((expires_at, id)));
        }

        let (price, visible, hidden, side) = (
            order.price(),
            order.visible_quantity(),
            order.hidden_quantity(),
            order.side(),
        );
        self.orders
            .insert(id, RestingLimitOrder::new(sequence_number, order));

        let queue_entry = QueueEntry::new(sequence_number, id);
        let levels = match side {
            Side::Buy => &mut self.bid_levels,
            Side::Sell => &mut self.ask_levels,
        };
        match levels.entry(price) {
            Entry::Occupied(mut e) => {
                e.get_mut().on_order_added(queue_entry, visible, hidden);
            }
            Entry::Vacant(e) => {
                let mut price_level = PriceLevel::new();
                price_level.on_order_added(queue_entry, visible, hidden);
                e.insert(price_level);
            }
        }
    }
}

impl PeggedBook {
    /// Add a pegged order to the order book
    pub(crate) fn add_order(
        &mut self,
        sequence_number: SequenceNumber,
        id: OrderId,
        order: PeggedOrder,
    ) {
        if let Some(expires_at) = order.expires_at() {
            self.expiration_queue.push(Reverse((expires_at, id)));
        }

        let (peg_reference, quantity, side) =
            (order.peg_reference(), order.quantity(), order.side());
        self.orders
            .insert(id, RestingPeggedOrder::new(sequence_number, order));

        let levels = match side {
            Side::Buy => &mut self.bid_levels,
            Side::Sell => &mut self.ask_levels,
        };
        levels[peg_reference.as_index()]
            .on_order_added(QueueEntry::new(sequence_number, id), quantity);
    }

    /// Remove a pegged order from the order book
    pub(crate) fn remove_order(&mut self, order_id: OrderId) -> Option<PeggedOrder> {
        let order = self.orders.remove(&order_id)?.into_order();

        let levels = match order.side() {
            Side::Buy => &mut self.bid_levels,
            Side::Sell => &mut self.ask_levels,
        };
        levels[order.peg_reference().as_index()].on_order_removed(order.quantity());

        Some(order)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::{
        OrderBook, OrderFlags, OrderId, PegReference, PeggedOrder, Price, Quantity, QuantityPolicy,
        TimeInForce,
    };

    #[test]
    fn test_add_limit_order() {
        let mut book = OrderBook::new("TEST");
        assert!(book.limit.bid_levels.is_empty());
        assert!(book.limit.ask_levels.is_empty());
        assert!(book.limit.orders.is_empty());

        let order = LimitOrder::new(
            Price(100),
            QuantityPolicy::Standard {
                quantity: Quantity(10),
            },
            OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
        );
        book.add_limit_order(SequenceNumber(0), OrderId(0), order.clone());
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
        assert_eq!(book.limit.orders.get(&OrderId(0)).unwrap().order(), &order);

        let order = LimitOrder::new(
            Price(100),
            QuantityPolicy::Standard {
                quantity: Quantity(10),
            },
            OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
        );
        book.add_limit_order(SequenceNumber(1), OrderId(1), order.clone());
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
        assert_eq!(book.limit.orders.get(&OrderId(1)).unwrap().order(), &order);

        let order = LimitOrder::new(
            Price(110),
            QuantityPolicy::Standard {
                quantity: Quantity(10),
            },
            OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
        );
        book.add_limit_order(SequenceNumber(2), OrderId(2), order.clone());
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
        assert_eq!(book.limit.orders.get(&OrderId(2)).unwrap().order(), &order);

        let order = LimitOrder::new(
            Price(105),
            QuantityPolicy::Standard {
                quantity: Quantity(10),
            },
            OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
        );
        book.add_limit_order(SequenceNumber(3), OrderId(3), order.clone());
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
        assert_eq!(book.limit.orders.get(&OrderId(3)).unwrap().order(), &order);
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
            PegReference::Primary,
            Quantity(10),
            OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
        );
        book.add_pegged_order(SequenceNumber(0), OrderId(0), order.clone());
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
        assert_eq!(book.pegged.orders.get(&OrderId(0)).unwrap().order(), &order);

        let order = PeggedOrder::new(
            PegReference::Market,
            Quantity(10),
            OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
        );
        book.add_pegged_order(SequenceNumber(1), OrderId(1), order.clone());
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
        assert_eq!(book.pegged.orders.get(&OrderId(1)).unwrap().order(), &order);

        let order = PeggedOrder::new(
            PegReference::MidPrice,
            Quantity(10),
            OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
        );
        book.add_pegged_order(SequenceNumber(2), OrderId(2), order.clone());
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
        assert_eq!(book.pegged.orders.get(&OrderId(2)).unwrap().order(), &order);

        let order = PeggedOrder::new(
            PegReference::Primary,
            Quantity(10),
            OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
        );
        book.add_pegged_order(SequenceNumber(3), OrderId(3), order.clone());
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
        assert_eq!(book.pegged.orders.get(&OrderId(3)).unwrap().order(), &order);

        let order = PeggedOrder::new(
            PegReference::MidPrice,
            Quantity(10),
            OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
        );
        book.add_pegged_order(SequenceNumber(4), OrderId(4), order.clone());
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
        assert_eq!(book.pegged.orders.get(&OrderId(4)).unwrap().order(), &order);

        let order = PeggedOrder::new(
            PegReference::MidPrice,
            Quantity(10),
            OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
        );
        book.add_pegged_order(SequenceNumber(5), OrderId(5), order.clone());
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
        assert_eq!(book.pegged.orders.get(&OrderId(5)).unwrap().order(), &order);
    }

    #[test]
    fn test_remove_limit_order_returns_order_when_present() {
        let mut book = OrderBook::new("TEST");
        let order = LimitOrder::new(
            Price(100),
            QuantityPolicy::Standard {
                quantity: Quantity(10),
            },
            OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
        );
        book.add_limit_order(SequenceNumber(0), OrderId(0), order.clone());
        assert_eq!(book.limit.orders.len(), 1);
        assert_eq!(
            book.limit
                .bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            1
        );

        let removed = book.remove_limit_order(SequenceNumber(1), OrderId(0));
        assert_eq!(removed.as_ref(), Some(&order));
        assert!(book.limit.orders.is_empty());
        assert!(!book.limit.bid_levels.contains_key(&Price(100)));
    }

    #[test]
    fn test_remove_limit_order_returns_none_when_absent() {
        let mut book = OrderBook::new("TEST");
        let removed = book.remove_limit_order(SequenceNumber(1000), OrderId(999));
        assert_eq!(removed, None);
    }

    #[test]
    fn test_remove_limit_order_one_of_many_at_same_price() {
        let mut book = OrderBook::new("TEST");
        for i in 0..3 {
            let order = LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            );
            book.add_limit_order(SequenceNumber(i), OrderId(i), order);
        }
        assert_eq!(book.limit.orders.len(), 3);
        assert_eq!(
            book.limit
                .bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            3
        );

        let removed = book.remove_limit_order(SequenceNumber(4), OrderId(1));
        assert!(removed.is_some());
        assert_eq!(book.limit.orders.len(), 2);
        assert_eq!(
            book.limit
                .bid_levels
                .get(&Price(100))
                .unwrap()
                .order_count(),
            2
        );
        assert!(!book.limit.orders.contains_key(&OrderId(1)));
    }

    #[test]
    fn test_remove_limit_order_cleans_up_empty_price_level() {
        let mut book = OrderBook::new("TEST");
        let order = LimitOrder::new(
            Price(100),
            QuantityPolicy::Standard {
                quantity: Quantity(10),
            },
            OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
        );
        book.add_limit_order(SequenceNumber(0), OrderId(0), order.clone());
        book.remove_limit_order(SequenceNumber(1), OrderId(0));
        assert!(book.limit.bid_levels.is_empty());
        assert!(book.limit.orders.is_empty());
    }

    #[test]
    fn test_remove_limit_order_leave_other_prices_unchanged() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_limit_order(
            SequenceNumber(1),
            OrderId(1),
            LimitOrder::new(
                Price(200),
                QuantityPolicy::Standard {
                    quantity: Quantity(5),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        assert_eq!(book.limit.bid_levels.len(), 2);

        book.remove_limit_order(SequenceNumber(2), OrderId(0));
        assert_eq!(book.limit.bid_levels.len(), 1);
        assert!(book.limit.bid_levels.contains_key(&Price(200)));
        assert_eq!(
            book.limit
                .bid_levels
                .get(&Price(200))
                .unwrap()
                .order_count(),
            1
        );
    }

    #[test]
    fn test_remove_pegged_order_returns_order_when_present() {
        let mut book = OrderBook::new("TEST");
        let order = PeggedOrder::new(
            PegReference::Primary,
            Quantity(10),
            OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
        );
        book.add_pegged_order(SequenceNumber(0), OrderId(0), order.clone());
        assert_eq!(book.pegged.orders.len(), 1);
        assert_eq!(
            book.pegged.bid_levels[PegReference::Primary.as_index()].order_count(),
            1
        );

        let removed = book.remove_pegged_order(OrderId(0));
        assert_eq!(removed.as_ref(), Some(&order));
        assert!(book.pegged.orders.is_empty());
        assert_eq!(
            book.pegged.bid_levels[PegReference::Primary.as_index()].order_count(),
            0
        );
    }

    #[test]
    fn test_remove_pegged_order_returns_none_when_absent() {
        let mut book = OrderBook::new("TEST");
        let removed = book.remove_pegged_order(OrderId(999));
        assert_eq!(removed, None);
    }

    #[test]
    fn test_remove_pegged_order_one_of_many_at_same_peg() {
        let mut book = OrderBook::new("TEST");
        for i in 0..3 {
            let order = PeggedOrder::new(
                PegReference::Market,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            );
            book.add_pegged_order(SequenceNumber(i), OrderId(i), order);
        }
        assert_eq!(book.pegged.orders.len(), 3);
        assert_eq!(
            book.pegged.bid_levels[PegReference::Market.as_index()].order_count(),
            3
        );

        let removed = book.remove_pegged_order(OrderId(1));
        assert!(removed.is_some());
        assert_eq!(book.pegged.orders.len(), 2);
        assert_eq!(
            book.pegged.bid_levels[PegReference::Market.as_index()].order_count(),
            2
        );
        assert!(!book.pegged.orders.contains_key(&OrderId(1)));
    }

    #[test]
    fn test_remove_pegged_order_leave_other_peg_levels_unchanged() {
        let mut book = OrderBook::new("TEST");
        book.add_pegged_order(
            SequenceNumber(0),
            OrderId(0),
            PeggedOrder::new(
                PegReference::Primary,
                Quantity(10),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            ),
        );
        book.add_pegged_order(
            SequenceNumber(1),
            OrderId(1),
            PeggedOrder::new(
                PegReference::MidPrice,
                Quantity(5),
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            ),
        );
        assert_eq!(
            book.pegged.bid_levels[PegReference::Primary.as_index()].order_count(),
            1
        );
        assert_eq!(
            book.pegged.ask_levels[PegReference::MidPrice.as_index()].order_count(),
            1
        );

        book.remove_pegged_order(OrderId(0));
        assert_eq!(
            book.pegged.bid_levels[PegReference::Primary.as_index()].order_count(),
            0
        );
        assert_eq!(
            book.pegged.ask_levels[PegReference::MidPrice.as_index()].order_count(),
            1
        );
        assert_eq!(book.pegged.orders.len(), 1);
    }
}
