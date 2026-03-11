use super::PriceLevel;
use crate::{LimitOrder, OrderId, Price, Quantity, Side, Timestamp};

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap, HashMap},
};

use serde::{Deserialize, Serialize};

/// Order book that manages orders and levels.
/// It supports adding, updating, cancelling, and matching orders.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LimitBook {
    /// Bid side price levels, stored in a ordered map with O(log N) ordering
    pub(crate) bid_levels: BTreeMap<Price, PriceLevel>,

    /// Ask side price levels, stored in a ordered map with O(log N) ordering
    pub(crate) ask_levels: BTreeMap<Price, PriceLevel>,

    /// Limit orders indexed by order ID for O(1) lookup
    pub(crate) orders: HashMap<OrderId, LimitOrder>,

    /// Queue of limit order IDs to be expired, stored in a min heap of tuples of
    /// (expires_at, order_id) with O(log N) ordering
    pub(crate) expiration_queue: BinaryHeap<Reverse<(Timestamp, OrderId)>>,
}

impl LimitBook {
    /// Create a new limit order book
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the bid side price levels
    pub fn bid_levels(&self) -> &BTreeMap<Price, PriceLevel> {
        &self.bid_levels
    }

    /// Get the ask side price levels
    pub fn ask_levels(&self) -> &BTreeMap<Price, PriceLevel> {
        &self.ask_levels
    }

    /// Get the limit orders indexed by order ID
    pub fn orders(&self) -> &HashMap<OrderId, LimitOrder> {
        &self.orders
    }

    /// Get the queue of limit order IDs to be expired
    pub fn expiration_queue(&self) -> &BinaryHeap<Reverse<(Timestamp, OrderId)>> {
        &self.expiration_queue
    }

    /// Get the best bid price, if any
    /// O(1) operation using the last key (highest price) in the BTreeMap
    pub fn best_bid_price(&self) -> Option<Price> {
        self.bid_levels.keys().next_back().copied()
    }

    /// Get the best ask price, if any
    /// O(1) operation using the first key (lowest price) in the BTreeMap
    pub fn best_ask_price(&self) -> Option<Price> {
        self.ask_levels.keys().next().copied()
    }

    /// Get the mid price (average of best bid and best ask)
    pub fn mid_price(&self) -> Option<f64> {
        let best_bid = self.best_bid_price()?;
        let best_ask = self.best_ask_price()?;
        Some((best_bid.as_f64() + best_ask.as_f64()) / 2.0)
    }

    /// Get the spread (difference between best bid and best ask)
    pub fn spread(&self) -> Option<u64> {
        let best_bid = self.best_bid_price()?;
        let best_ask = self.best_ask_price()?;
        Some(best_ask - best_bid)
    }

    /// Get the best bid volume, if not empty
    pub fn best_bid_volume(&self) -> Option<Quantity> {
        self.bid_levels
            .values()
            .next_back()
            .map(|level| level.total_quantity())
    }

    /// Get the best ask volume, if not empty
    pub fn best_ask_volume(&self) -> Option<Quantity> {
        self.ask_levels
            .values()
            .next()
            .map(|level| level.total_quantity())
    }

    /// Check if the side is empty
    pub fn is_side_empty(&self, side: Side) -> bool {
        match side {
            Side::Buy => self.bid_levels.is_empty(),
            Side::Sell => self.ask_levels.is_empty(),
        }
    }

    /// Check if there is a crossable order at the given limit price
    pub fn has_crossable_order(&self, taker_side: Side, limit_price: Price) -> bool {
        match taker_side {
            Side::Buy => self.best_ask_price().is_some_and(|ask| limit_price >= ask),
            Side::Sell => self.best_bid_price().is_some_and(|bid| limit_price <= bid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_level(visible_qty: Quantity) -> PriceLevel {
        let mut level = PriceLevel::new();
        level.visible_quantity = visible_qty;
        level
    }

    fn book_with_bids_and_asks(
        bids: &[(Price, Quantity)],
        asks: &[(Price, Quantity)],
    ) -> LimitBook {
        let mut book = LimitBook::new();
        for &(price, qty) in bids {
            book.bid_levels.insert(price, make_level(qty));
        }
        for &(price, qty) in asks {
            book.ask_levels.insert(price, make_level(qty));
        }
        book
    }

    // --- best_bid_price ---

    #[test]
    fn best_bid_price_empty() {
        let book = LimitBook::new();
        assert_eq!(book.best_bid_price(), None);
    }

    #[test]
    fn best_bid_price_single() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[]);
        assert_eq!(book.best_bid_price(), Some(Price(100)));
    }

    #[test]
    fn best_bid_price_returns_highest() {
        let book = book_with_bids_and_asks(
            &[
                (Price(90), Quantity(5)),
                (Price(100), Quantity(10)),
                (Price(95), Quantity(7)),
            ],
            &[],
        );
        assert_eq!(book.best_bid_price(), Some(Price(100)));
    }

    // --- best_ask_price ---

    #[test]
    fn best_ask_price_empty() {
        let book = LimitBook::new();
        assert_eq!(book.best_ask_price(), None);
    }

    #[test]
    fn best_ask_price_single() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(10))]);
        assert_eq!(book.best_ask_price(), Some(Price(200)));
    }

    #[test]
    fn best_ask_price_returns_lowest() {
        let book = book_with_bids_and_asks(
            &[],
            &[
                (Price(210), Quantity(5)),
                (Price(200), Quantity(10)),
                (Price(205), Quantity(7)),
            ],
        );
        assert_eq!(book.best_ask_price(), Some(Price(200)));
    }

    // --- mid_price ---

    #[test]
    fn mid_price_empty_book() {
        let book = LimitBook::new();
        assert_eq!(book.mid_price(), None);
    }

    #[test]
    fn mid_price_only_bids() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[]);
        assert_eq!(book.mid_price(), None);
    }

    #[test]
    fn mid_price_only_asks() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(10))]);
        assert_eq!(book.mid_price(), None);
    }

    #[test]
    fn mid_price_both_sides() {
        let book =
            book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[(Price(200), Quantity(10))]);
        assert_eq!(book.mid_price(), Some(150.0));
    }

    #[test]
    fn mid_price_tight_spread() {
        let book =
            book_with_bids_and_asks(&[(Price(99), Quantity(10))], &[(Price(101), Quantity(10))]);
        assert_eq!(book.mid_price(), Some(100.0));
    }

    #[test]
    fn mid_price_odd_spread() {
        let book =
            book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[(Price(101), Quantity(10))]);
        assert_eq!(book.mid_price(), Some(100.5));
    }

    // --- spread ---

    #[test]
    fn spread_empty_book() {
        let book = LimitBook::new();
        assert_eq!(book.spread(), None);
    }

    #[test]
    fn spread_only_bids() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[]);
        assert_eq!(book.spread(), None);
    }

    #[test]
    fn spread_only_asks() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(10))]);
        assert_eq!(book.spread(), None);
    }

    #[test]
    fn spread_both_sides() {
        let book =
            book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[(Price(200), Quantity(10))]);
        assert_eq!(book.spread(), Some(100));
    }

    #[test]
    fn spread_one_tick() {
        let book =
            book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[(Price(101), Quantity(10))]);
        assert_eq!(book.spread(), Some(1));
    }

    // --- best_bid_volume ---

    #[test]
    fn best_bid_volume_empty() {
        let book = LimitBook::new();
        assert_eq!(book.best_bid_volume(), None);
    }

    #[test]
    fn best_bid_volume_single_level() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(50))], &[]);
        assert_eq!(book.best_bid_volume(), Some(Quantity(50)));
    }

    #[test]
    fn best_bid_volume_multiple_levels() {
        let book = book_with_bids_and_asks(
            &[
                (Price(90), Quantity(30)),
                (Price(100), Quantity(50)),
                (Price(95), Quantity(20)),
            ],
            &[],
        );
        assert_eq!(book.best_bid_volume(), Some(Quantity(50)));
    }

    // --- best_ask_volume ---

    #[test]
    fn best_ask_volume_empty() {
        let book = LimitBook::new();
        assert_eq!(book.best_ask_volume(), None);
    }

    #[test]
    fn best_ask_volume_single_level() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(40))]);
        assert_eq!(book.best_ask_volume(), Some(Quantity(40)));
    }

    #[test]
    fn best_ask_volume_multiple_levels() {
        let book = book_with_bids_and_asks(
            &[],
            &[
                (Price(200), Quantity(40)),
                (Price(210), Quantity(60)),
                (Price(205), Quantity(25)),
            ],
        );
        assert_eq!(book.best_ask_volume(), Some(Quantity(40)));
    }

    // --- is_side_empty ---

    #[test]
    fn is_side_empty_both_empty() {
        let book = LimitBook::new();
        assert!(book.is_side_empty(Side::Buy));
        assert!(book.is_side_empty(Side::Sell));
    }

    #[test]
    fn is_side_empty_with_bids() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[]);
        assert!(!book.is_side_empty(Side::Buy));
        assert!(book.is_side_empty(Side::Sell));
    }

    #[test]
    fn is_side_empty_with_asks() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(10))]);
        assert!(book.is_side_empty(Side::Buy));
        assert!(!book.is_side_empty(Side::Sell));
    }

    #[test]
    fn is_side_empty_both_sides() {
        let book =
            book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[(Price(200), Quantity(10))]);
        assert!(!book.is_side_empty(Side::Buy));
        assert!(!book.is_side_empty(Side::Sell));
    }

    // --- has_crossable_order ---

    #[test]
    fn has_crossable_order_empty_book() {
        let book = LimitBook::new();
        assert!(!book.has_crossable_order(Side::Buy, Price(100)));
        assert!(!book.has_crossable_order(Side::Sell, Price(100)));
    }

    #[test]
    fn has_crossable_buy_at_ask() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(10))]);
        assert!(book.has_crossable_order(Side::Buy, Price(200)));
    }

    #[test]
    fn has_crossable_buy_above_ask() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(10))]);
        assert!(book.has_crossable_order(Side::Buy, Price(250)));
    }

    #[test]
    fn has_crossable_buy_below_ask() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(10))]);
        assert!(!book.has_crossable_order(Side::Buy, Price(199)));
    }

    #[test]
    fn has_crossable_sell_at_bid() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[]);
        assert!(book.has_crossable_order(Side::Sell, Price(100)));
    }

    #[test]
    fn has_crossable_sell_below_bid() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[]);
        assert!(book.has_crossable_order(Side::Sell, Price(50)));
    }

    #[test]
    fn has_crossable_sell_above_bid() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[]);
        assert!(!book.has_crossable_order(Side::Sell, Price(101)));
    }

    #[test]
    fn has_crossable_order_no_opposite_side() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[]);
        assert!(!book.has_crossable_order(Side::Buy, Price(200)));

        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(10))]);
        assert!(!book.has_crossable_order(Side::Sell, Price(100)));
    }
}
