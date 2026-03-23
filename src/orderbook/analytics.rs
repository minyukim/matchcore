//! Analytics for the order book

mod depth_statistics;
mod market_impact;

pub use depth_statistics::*;
pub use market_impact::*;

use super::{LimitBook, OrderBook};
use crate::{Notional, PegReference, Price, Quantity, Side};

impl LimitBook {
    /// Get the best bid price and size, if exists
    pub fn best_bid(&self) -> Option<(Price, Quantity)> {
        self.bids
            .iter()
            .next_back()
            .map(|(price, level_id)| (*price, self.levels[*level_id].total_quantity()))
    }

    /// Get the best ask price and size, if exists
    pub fn best_ask(&self) -> Option<(Price, Quantity)> {
        self.asks
            .iter()
            .next()
            .map(|(price, level_id)| (*price, self.levels[*level_id].total_quantity()))
    }

    /// Get the best bid price, if exists
    pub fn best_bid_price(&self) -> Option<Price> {
        self.bids.keys().next_back().copied()
    }

    /// Get the best ask price, if exists
    pub fn best_ask_price(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    /// Get the best bid size, if exists
    pub fn best_bid_size(&self) -> Option<Quantity> {
        self.bids
            .values()
            .next_back()
            .map(|level_id| self.levels[*level_id].total_quantity())
    }

    /// Get the best ask size, if exists
    pub fn best_ask_size(&self) -> Option<Quantity> {
        self.asks
            .values()
            .next()
            .map(|level_id| self.levels[*level_id].total_quantity())
    }

    /// Check if the side is empty
    pub fn is_side_empty(&self, side: Side) -> bool {
        match side {
            Side::Buy => self.bids.is_empty(),
            Side::Sell => self.asks.is_empty(),
        }
    }

    /// Check if there is a crossable order at the given limit price
    pub fn has_crossable_order(&self, taker_side: Side, limit_price: Price) -> bool {
        match taker_side {
            Side::Buy => self.best_ask_price().is_some_and(|ask| limit_price >= ask),
            Side::Sell => self.best_bid_price().is_some_and(|bid| limit_price <= bid),
        }
    }

    /// Get the spread (difference between best bid and best ask)
    pub fn spread(&self) -> Option<u64> {
        let best_bid = self.best_bid_price()?;
        let best_ask = self.best_ask_price()?;
        Some(best_ask - best_bid)
    }

    /// Get the mid price (average of best bid and best ask)
    pub fn mid_price(&self) -> Option<f64> {
        let best_bid = self.best_bid_price()?;
        let best_ask = self.best_ask_price()?;
        Some((best_bid.as_f64() + best_ask.as_f64()) / 2.0)
    }

    /// Calculate the micro price, which weights the best bid and ask by the opposite side's liquidity
    pub fn micro_price(&self) -> Option<f64> {
        let (best_bid_price, best_bid_size) = self.best_bid()?;
        let (best_ask_price, best_ask_size) = self.best_ask()?;

        let total_size = best_bid_size.saturating_add(best_ask_size);

        if total_size.is_zero() {
            return None;
        }

        // micro_price = (ask_price * bid_size + bid_price * ask_size) / (bid_size + ask_size)
        let numerator = (best_ask_price * best_bid_size) + (best_bid_price * best_ask_size);
        let denominator = total_size;

        Some(numerator / denominator)
    }

    /// Get the bid size for the first N price levels
    pub fn bid_size(&self, n_levels: usize) -> Quantity {
        self.bids
            .values()
            .rev()
            .take(n_levels)
            .map(|level_id| self.levels[*level_id].total_quantity())
            .sum::<Quantity>()
    }

    /// Get the ask size for the first N price levels
    pub fn ask_size(&self, n_levels: usize) -> Quantity {
        self.asks
            .values()
            .take(n_levels)
            .map(|level_id| self.levels[*level_id].total_quantity())
            .sum::<Quantity>()
    }

    /// Check if the order book is thin at the given threshold and number of levels
    pub fn is_thin_book(&self, threshold: Quantity, n_levels: usize) -> bool {
        let bid_size = self.bid_size(n_levels);
        let ask_size = self.ask_size(n_levels);

        bid_size < threshold || ask_size < threshold
    }

    /// Calculate the order book imbalance ratio for the top N levels
    pub fn order_book_imbalance(&self, n_levels: usize) -> f64 {
        let bid_size = self.bid_size(n_levels);
        let ask_size = self.ask_size(n_levels);

        let total_size = bid_size.saturating_add(ask_size);

        if total_size.is_zero() {
            return 0.0;
        }

        (bid_size.as_f64() - ask_size.as_f64()) / total_size.as_f64()
    }

    /// Compute the depth statistics of price levels (0 n_levels means all levels)
    pub fn depth_statistics(&self, side: Side, n_levels: usize) -> DepthStatistics {
        DepthStatistics::compute(self, side, n_levels)
    }
}

impl OrderBook {
    /// Get the best bid price and size, if exists
    pub fn best_bid(&self) -> Option<(Price, Quantity)> {
        self.limit.best_bid()
    }

    /// Get the best ask price and size, if exists
    pub fn best_ask(&self) -> Option<(Price, Quantity)> {
        self.limit.best_ask()
    }

    /// Get the best bid price, if exists
    pub fn best_bid_price(&self) -> Option<Price> {
        self.limit.best_bid_price()
    }

    /// Get the best ask price, if exists
    pub fn best_ask_price(&self) -> Option<Price> {
        self.limit.best_ask_price()
    }

    /// Get the best bid size, if exists
    pub fn best_bid_size(&self) -> Option<Quantity> {
        self.limit.best_bid_size()
    }

    /// Get the best ask size, if exists
    pub fn best_ask_size(&self) -> Option<Quantity> {
        self.limit.best_ask_size()
    }

    /// Check if the side is empty
    pub fn is_side_empty(&self, side: Side) -> bool {
        self.limit.is_side_empty(side)
    }

    /// Check if there is a crossable order at the given limit price
    pub fn has_crossable_order(&self, taker_side: Side, limit_price: Price) -> bool {
        self.limit.has_crossable_order(taker_side, limit_price)
    }

    /// Get the spread (difference between best bid and best ask)
    pub fn spread(&self) -> Option<u64> {
        self.limit.spread()
    }

    /// Get the mid price (average of best bid and best ask)
    pub fn mid_price(&self) -> Option<f64> {
        self.limit.mid_price()
    }

    /// Calculate the micro price, which weights the best bid and ask by the opposite side's liquidity
    pub fn micro_price(&self) -> Option<f64> {
        self.limit.micro_price()
    }

    /// Get the bid size for the first N price levels
    pub fn bid_size(&self, n_levels: usize) -> Quantity {
        self.limit.bid_size(n_levels)
    }

    /// Get the ask size for the first N price levels
    pub fn ask_size(&self, n_levels: usize) -> Quantity {
        self.limit.ask_size(n_levels)
    }

    /// Check if the order book is thin at the given threshold and number of levels
    pub fn is_thin_book(&self, threshold: Quantity, n_levels: usize) -> bool {
        self.limit.is_thin_book(threshold, n_levels)
    }

    /// Calculate the order book imbalance ratio for the top N levels
    pub fn order_book_imbalance(&self, n_levels: usize) -> f64 {
        self.limit.order_book_imbalance(n_levels)
    }

    /// Compute the depth statistics of price levels (0 n_levels means all levels)
    pub fn depth_statistics(&self, side: Side, n_levels: usize) -> DepthStatistics {
        self.limit.depth_statistics(side, n_levels)
    }

    /// Compute the buy and sell pressure of the order book
    pub fn buy_sell_pressure(&self) -> (Quantity, Quantity) {
        let buy_limit_pressure = self.bid_size(usize::MAX);
        let sell_limit_pressure = self.ask_size(usize::MAX);
        let buy_peg_pressure = self
            .pegged
            .bid_levels
            .iter()
            .map(|level| level.quantity())
            .sum();
        let sell_peg_pressure = self
            .pegged
            .ask_levels
            .iter()
            .map(|level| level.quantity())
            .sum();
        (
            buy_limit_pressure.saturating_add(buy_peg_pressure),
            sell_limit_pressure.saturating_add(sell_peg_pressure),
        )
    }

    /// Find the price where cumulative depth reaches the target quantity
    /// Return `None` if the target depth cannot be reached
    pub fn price_at_depth(&self, side: Side, depth: Quantity) -> Option<Price> {
        // MidPrice peg level is active if the spread is less than or equal to 1
        let mid_active = self.spread().is_some_and(|spread| spread <= 1);

        let mut cumulative = Quantity(0);

        match side {
            Side::Buy => {
                if mid_active {
                    cumulative = cumulative.saturating_add(
                        self.pegged.bid_levels[PegReference::MidPrice.as_index()].quantity(),
                    );
                }
                // Primary peg level is always active
                cumulative = cumulative.saturating_add(
                    self.pegged.bid_levels[PegReference::Primary.as_index()].quantity(),
                );

                // Iterate over the limit bid price levels
                for (price, level_id) in self.limit.bids.iter().rev() {
                    let level = &self.limit.levels[*level_id];
                    cumulative = cumulative.saturating_add(level.total_quantity());
                    if cumulative >= depth {
                        return Some(*price);
                    }
                }

                None
            }
            Side::Sell => {
                if mid_active {
                    cumulative = cumulative.saturating_add(
                        self.pegged.ask_levels[PegReference::MidPrice.as_index()].quantity(),
                    );
                }
                // Primary peg level is always active
                cumulative = cumulative.saturating_add(
                    self.pegged.ask_levels[PegReference::Primary.as_index()].quantity(),
                );

                // Iterate over the limit ask price levels
                for (price, level_id) in self.limit.asks.iter() {
                    let level = &self.limit.levels[*level_id];
                    cumulative = cumulative.saturating_add(level.total_quantity());
                    if cumulative >= depth {
                        return Some(*price);
                    }
                }

                None
            }
        }
    }

    /// Calculate the volume-weighted average price (VWAP) for a given quantity
    /// Return `None` if the given quantity is zero or cannot be filled
    pub fn vwap(&self, taker_side: Side, quantity: Quantity) -> Option<f64> {
        if quantity.is_zero() {
            return None;
        }

        // MidPrice peg level is active if the spread is less than or equal to 1
        let mid_active = self.spread().is_some_and(|spread| spread <= 1);

        let mut remaining = quantity;
        let mut total_cost = Notional(0);
        let mut total_filled = Quantity(0);

        match taker_side {
            Side::Buy => {
                // No liquidity
                let best_ask = self.best_ask_price()?;

                if mid_active {
                    let available =
                        self.pegged.ask_levels[PegReference::MidPrice.as_index()].quantity();
                    let fill_qty = remaining.min(available);
                    total_cost = total_cost.saturating_add(best_ask * fill_qty);
                    total_filled = total_filled.saturating_add(fill_qty);
                    remaining = remaining.saturating_sub(fill_qty);
                    if remaining.is_zero() {
                        return Some(total_cost / total_filled);
                    }
                }
                // Primary peg level is always active
                let available = self.pegged.ask_levels[PegReference::Primary.as_index()].quantity();
                let fill_qty = remaining.min(available);
                total_cost = total_cost.saturating_add(best_ask * fill_qty);
                total_filled = total_filled.saturating_add(fill_qty);
                remaining = remaining.saturating_sub(fill_qty);
                if remaining.is_zero() {
                    return Some(total_cost / total_filled);
                }

                // Iterate over the limit ask price levels
                for (price, level_id) in self.limit.asks.iter() {
                    let level = &self.limit.levels[*level_id];
                    let available = level.total_quantity();
                    let fill_qty = remaining.min(available);
                    total_cost = total_cost.saturating_add(*price * fill_qty);
                    total_filled = total_filled.saturating_add(fill_qty);
                    remaining = remaining.saturating_sub(fill_qty);
                    if remaining.is_zero() {
                        return Some(total_cost / total_filled);
                    }
                }

                None
            }
            Side::Sell => {
                // No liquidity
                let best_bid = self.best_bid_price()?;

                if mid_active {
                    let available =
                        self.pegged.bid_levels[PegReference::MidPrice.as_index()].quantity();
                    let fill_qty = remaining.min(available);
                    total_cost = total_cost.saturating_add(best_bid * fill_qty);
                    total_filled = total_filled.saturating_add(fill_qty);
                    remaining = remaining.saturating_sub(fill_qty);
                    if remaining.is_zero() {
                        return Some(total_cost / total_filled);
                    }
                }
                // Primary peg level is always active
                let available = self.pegged.bid_levels[PegReference::Primary.as_index()].quantity();
                let fill_qty = remaining.min(available);
                total_cost = total_cost.saturating_add(best_bid * fill_qty);
                total_filled = total_filled.saturating_add(fill_qty);
                remaining = remaining.saturating_sub(fill_qty);
                if remaining.is_zero() {
                    return Some(total_cost / total_filled);
                }

                // Iterate over the limit bid price levels
                for (price, level_id) in self.limit.bids.iter().rev() {
                    let level = &self.limit.levels[*level_id];
                    let available = level.total_quantity();
                    let fill_qty = remaining.min(available);
                    total_cost = total_cost.saturating_add(*price * fill_qty);
                    total_filled = total_filled.saturating_add(fill_qty);
                    remaining = remaining.saturating_sub(fill_qty);
                    if remaining.is_zero() {
                        return Some(total_cost / total_filled);
                    }
                }

                None
            }
        }
    }

    /// Compute the market impact of a market order
    pub fn market_impact(&self, taker_side: Side, quantity: Quantity) -> MarketImpact {
        MarketImpact::compute(self, taker_side, quantity)
    }
}

#[cfg(test)]
mod tests_limit_book {
    use super::*;
    use crate::PriceLevel;

    const EPS: f64 = 1e-6;

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
            let level = make_level(qty);
            let level_id = book.levels.insert(level);
            book.bids.insert(price, level_id);
        }
        for &(price, qty) in asks {
            let level = make_level(qty);
            let level_id = book.levels.insert(level);
            book.asks.insert(price, level_id);
        }
        book
    }

    /// Bid 100 (qty 50), Bid 99 (qty 30)
    /// Ask 101 (qty 40), Ask 102 (qty 60)
    fn basic_book() -> LimitBook {
        book_with_bids_and_asks(
            &[(Price(100), Quantity(50)), (Price(99), Quantity(30))],
            &[(Price(101), Quantity(40)), (Price(102), Quantity(60))],
        )
    }

    // ==================== best_bid ====================

    #[test]
    fn best_bid_empty() {
        let book = LimitBook::new();
        assert_eq!(book.best_bid(), None);
    }

    #[test]
    fn best_bid_single() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[]);
        assert_eq!(book.best_bid(), Some((Price(100), Quantity(10))));
    }

    #[test]
    fn best_bid_multiple() {
        let book = book_with_bids_and_asks(
            &[
                (Price(90), Quantity(5)),
                (Price(100), Quantity(10)),
                (Price(95), Quantity(7)),
            ],
            &[],
        );
        assert_eq!(book.best_bid(), Some((Price(100), Quantity(10))));
    }

    // ==================== best_ask ====================

    #[test]
    fn best_ask_empty() {
        let book = LimitBook::new();
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn best_ask_single() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(10))]);
        assert_eq!(book.best_ask(), Some((Price(200), Quantity(10))));
    }

    #[test]
    fn best_ask_multiple() {
        let book = book_with_bids_and_asks(
            &[],
            &[
                (Price(210), Quantity(5)),
                (Price(200), Quantity(10)),
                (Price(205), Quantity(7)),
            ],
        );
        assert_eq!(book.best_ask(), Some((Price(200), Quantity(10))));
    }

    // ==================== best_bid_price ====================

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

    // ==================== best_ask_price ====================

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

    // ==================== best_bid_size ====================

    #[test]
    fn best_bid_size_empty() {
        let book = LimitBook::new();
        assert_eq!(book.best_bid_size(), None);
    }

    #[test]
    fn best_bid_size_single_level() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(50))], &[]);
        assert_eq!(book.best_bid_size(), Some(Quantity(50)));
    }

    #[test]
    fn best_bid_size_multiple_levels() {
        let book = book_with_bids_and_asks(
            &[
                (Price(90), Quantity(30)),
                (Price(100), Quantity(50)),
                (Price(95), Quantity(20)),
            ],
            &[],
        );
        assert_eq!(book.best_bid_size(), Some(Quantity(50)));
    }

    // ==================== best_ask_size ====================

    #[test]
    fn best_ask_size_empty() {
        let book = LimitBook::new();
        assert_eq!(book.best_ask_size(), None);
    }

    #[test]
    fn best_ask_size_single_level() {
        let book = book_with_bids_and_asks(&[], &[(Price(200), Quantity(40))]);
        assert_eq!(book.best_ask_size(), Some(Quantity(40)));
    }

    #[test]
    fn best_ask_size_multiple_levels() {
        let book = book_with_bids_and_asks(
            &[],
            &[
                (Price(200), Quantity(40)),
                (Price(210), Quantity(60)),
                (Price(205), Quantity(25)),
            ],
        );
        assert_eq!(book.best_ask_size(), Some(Quantity(40)));
    }

    // ==================== is_side_empty ====================

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

    // ==================== has_crossable_order ====================

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

    // ==================== spread ====================

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

    // ==================== mid_price ====================

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

    // ==================== micro_price ====================

    #[test]
    fn micro_price_empty_book() {
        assert!(LimitBook::new().micro_price().is_none());
    }

    #[test]
    fn micro_price_balanced_sizes() {
        let book = book_with_bids_and_asks(
            &[(Price(100), Quantity(100))],
            &[(Price(102), Quantity(100))],
        );

        // Equal sizes => micro_price = midpoint = (100 + 102) / 2 = 101
        assert!((book.micro_price().unwrap() - 101.0).abs() < EPS);
    }

    #[test]
    fn micro_price_imbalanced_toward_bid() {
        let book = book_with_bids_and_asks(
            &[(Price(100), Quantity(300))],
            &[(Price(102), Quantity(100))],
        );

        // micro = (102 * 300 + 100 * 100) / 400 = (30600 + 10000) / 400 = 101.5
        assert!((book.micro_price().unwrap() - 101.5).abs() < EPS);
    }

    #[test]
    fn micro_price_imbalanced_toward_ask() {
        let book = book_with_bids_and_asks(
            &[(Price(100), Quantity(100))],
            &[(Price(102), Quantity(300))],
        );

        // micro = (102 * 100 + 100 * 300) / 400 = (10200 + 30000) / 400 = 100.5
        assert!((book.micro_price().unwrap() - 100.5).abs() < EPS);
    }

    // ==================== bid_size and ask_size ====================

    #[test]
    fn bid_ask_sizes_empty_book() {
        let book = LimitBook::new();
        assert_eq!(book.bid_size(5), Quantity(0));
        assert_eq!(book.ask_size(5), Quantity(0));
    }

    #[test]
    fn bid_ask_sizes_n_levels_zero() {
        let book =
            book_with_bids_and_asks(&[(Price(100), Quantity(10))], &[(Price(102), Quantity(10))]);
        assert_eq!(book.bid_size(0), Quantity(0));
        assert_eq!(book.ask_size(0), Quantity(0));
    }

    #[test]
    fn bid_ask_sizes_single_level() {
        let book = basic_book();
        assert_eq!(book.bid_size(1), Quantity(50));
        assert_eq!(book.ask_size(1), Quantity(40));
    }

    #[test]
    fn bid_ask_sizes_multiple_levels() {
        let book = basic_book();
        assert_eq!(book.bid_size(2), Quantity(80));
        assert_eq!(book.ask_size(2), Quantity(100));
    }

    #[test]
    fn bid_ask_sizes_exceeding_available_levels() {
        let book = basic_book();
        assert_eq!(book.bid_size(100), Quantity(80));
        assert_eq!(book.ask_size(100), Quantity(100));
    }

    // ==================== is_thin_book ====================

    #[test]
    fn is_thin_book_empty() {
        assert!(LimitBook::new().is_thin_book(Quantity(1), 1));
    }

    #[test]
    fn is_thin_book_sufficient_depth() {
        let book = basic_book();
        // Bid top 2: 80, Ask top 2: 100. Threshold 50 => not thin
        assert!(!book.is_thin_book(Quantity(50), 2));
    }

    #[test]
    fn is_thin_book_one_side_thin() {
        let book = basic_book();
        // Bid top 1: 50, Ask top 1: 40. Threshold 45 => ask is thin
        assert!(book.is_thin_book(Quantity(45), 1));
    }

    #[test]
    fn is_thin_book_both_sides_sufficient() {
        let book = basic_book();
        // Bid top 1: 50, Ask top 1: 40. Threshold 40 => not thin (both >= 40)
        assert!(!book.is_thin_book(Quantity(40), 1));
    }

    #[test]
    fn is_thin_book_threshold_zero() {
        assert!(!LimitBook::new().is_thin_book(Quantity(0), 1));
    }

    // ==================== order_book_imbalance ====================

    #[test]
    fn order_book_imbalance_empty_book() {
        assert_eq!(LimitBook::new().order_book_imbalance(5), 0.0);
    }

    #[test]
    fn order_book_imbalance_balanced() {
        let book = book_with_bids_and_asks(
            &[(Price(100), Quantity(100))],
            &[(Price(101), Quantity(100))],
        );
        assert!((book.order_book_imbalance(1) - 0.0).abs() < EPS);
    }

    #[test]
    fn order_book_imbalance_all_bids() {
        let book = book_with_bids_and_asks(&[(Price(100), Quantity(100))], &[]);
        assert!((book.order_book_imbalance(1) - 1.0).abs() < EPS);
    }

    #[test]
    fn order_book_imbalance_all_asks() {
        let book = book_with_bids_and_asks(&[], &[(Price(101), Quantity(100))]);
        assert!((book.order_book_imbalance(1) - (-1.0)).abs() < EPS);
    }

    #[test]
    fn order_book_imbalance_multiple_levels() {
        let book = basic_book();
        // 1 level: bid=50, ask=40 => (50-40)/90
        let imb1 = book.order_book_imbalance(1);
        assert!((imb1 - 10.0 / 90.0).abs() < EPS);

        // 2 levels: bid=80, ask=100 => (80-100)/180
        let imb2 = book.order_book_imbalance(2);
        assert!((imb2 - (-20.0 / 180.0)).abs() < EPS);
    }

    #[test]
    fn order_book_imbalance_n_levels_zero() {
        let book = basic_book();
        assert_eq!(book.order_book_imbalance(0), 0.0);
    }

    // ==================== depth_statistics ====================

    #[test]
    fn depth_statistics_empty_book() {
        let stats = LimitBook::new().depth_statistics(Side::Buy, 5);
        assert!(stats.is_empty());
        assert_eq!(stats.n_analyzed_levels(), 0);
        assert_eq!(stats.total_size(), Quantity(0));
        assert_eq!(stats.min_level_size(), Quantity(0));
        assert_eq!(stats.max_level_size(), Quantity(0));
    }

    #[test]
    fn depth_statistics_single_level() {
        let book = book_with_bids_and_asks(&[(Price(50), Quantity(100))], &[]);
        let stats = book.depth_statistics(Side::Buy, 1);
        assert_eq!(stats.n_analyzed_levels(), 1);
        assert_eq!(stats.total_value(), Notional(5000));
        assert_eq!(stats.total_size(), Quantity(100));
        assert_eq!(stats.min_level_size(), Quantity(100));
        assert_eq!(stats.max_level_size(), Quantity(100));
        assert!((stats.average_level_size() - 100.0).abs() < EPS);
        assert!((stats.std_dev_level_size() - 0.0).abs() < EPS);
        assert!((stats.vwap() - 50.0).abs() < EPS);
    }

    #[test]
    fn depth_statistics_multiple_levels() {
        let book = basic_book();
        let stats = book.depth_statistics(Side::Buy, 2);
        assert_eq!(stats.n_analyzed_levels(), 2);
        assert_eq!(stats.total_value(), Notional(7970)); // 50 * 100 + 30 * 99
        assert_eq!(stats.total_size(), Quantity(80)); // 50 + 30
        assert_eq!(stats.min_level_size(), Quantity(30));
        assert_eq!(stats.max_level_size(), Quantity(50));
        assert!((stats.average_level_size() - 40.0).abs() < EPS);
    }

    #[test]
    fn depth_statistics_sell_side() {
        let book = basic_book();
        let stats = book.depth_statistics(Side::Sell, 2);
        assert_eq!(stats.n_analyzed_levels(), 2);
        assert_eq!(stats.total_value(), Notional(10160)); // 40 * 101 + 60 * 102
        assert_eq!(stats.total_size(), Quantity(100)); // 40 + 60
        assert_eq!(stats.min_level_size(), Quantity(40));
        assert_eq!(stats.max_level_size(), Quantity(60));
        assert!((stats.average_level_size() - 50.0).abs() < EPS);
    }

    #[test]
    fn depth_statistics_zero_levels_means_all() {
        let book = basic_book();
        let stats = book.depth_statistics(Side::Buy, 0);
        assert_eq!(stats.n_analyzed_levels(), 2);
        assert_eq!(stats.total_size(), Quantity(80));
    }
}

#[cfg(test)]
mod tests_order_book {
    use super::*;
    use crate::{
        LimitOrder, OrderFlags, OrderId, PegReference, PeggedOrder, Price, Quantity,
        QuantityPolicy, SequenceNumber, Side, TimeInForce,
    };

    const EPS: f64 = 1e-9;

    fn empty_book() -> OrderBook {
        OrderBook::new("TEST")
    }

    fn standard(price: u64, qty: u64, side: Side) -> LimitOrder {
        LimitOrder::new(
            Price(price),
            QuantityPolicy::Standard {
                quantity: Quantity(qty),
            },
            OrderFlags::new(side, false, TimeInForce::Gtc),
        )
    }

    fn iceberg(price: u64, visible: u64, hidden: u64, side: Side) -> LimitOrder {
        LimitOrder::new(
            Price(price),
            QuantityPolicy::Iceberg {
                visible_quantity: Quantity(visible),
                hidden_quantity: Quantity(hidden),
                replenish_quantity: Quantity(visible),
            },
            OrderFlags::new(side, false, TimeInForce::Gtc),
        )
    }

    fn pegged(reference: PegReference, qty: u64, side: Side) -> PeggedOrder {
        PeggedOrder::new(
            reference,
            Quantity(qty),
            OrderFlags::new(side, false, TimeInForce::Gtc),
        )
    }

    /// Bid 100 (qty 50), Bid 99 (qty 30)
    /// Ask 101 (qty 40), Ask 102 (qty 60)
    fn basic_book() -> OrderBook {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(SequenceNumber(0), OrderId(0), standard(100, 50, Side::Buy));
        book.add_limit_order(SequenceNumber(1), OrderId(1), standard(99, 30, Side::Buy));
        book.add_limit_order(SequenceNumber(2), OrderId(2), standard(101, 40, Side::Sell));
        book.add_limit_order(SequenceNumber(3), OrderId(3), standard(102, 60, Side::Sell));
        book
    }

    /// Same as basic_book but with hidden quantities:
    /// Bid 100 (vis 50, hid 10), Bid 99 (vis 30, hid 20),
    /// Ask 101 (vis 40, hid 5), Ask 102 (vis 60, hid 15)
    fn book_with_hidden() -> OrderBook {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(
            SequenceNumber(0),
            OrderId(0),
            iceberg(100, 50, 10, Side::Buy),
        );
        book.add_limit_order(
            SequenceNumber(1),
            OrderId(1),
            iceberg(99, 30, 20, Side::Buy),
        );
        book.add_limit_order(
            SequenceNumber(2),
            OrderId(2),
            iceberg(101, 40, 5, Side::Sell),
        );
        book.add_limit_order(
            SequenceNumber(3),
            OrderId(3),
            iceberg(102, 60, 15, Side::Sell),
        );
        book
    }

    /// basic_book + Primary peg bid 20, Primary peg ask 25
    fn book_with_pegs() -> OrderBook {
        let mut book = basic_book();
        book.add_pegged_order(
            SequenceNumber(10),
            OrderId(10),
            pegged(PegReference::Primary, 20, Side::Buy),
        );
        book.add_pegged_order(
            SequenceNumber(11),
            OrderId(11),
            pegged(PegReference::Primary, 25, Side::Sell),
        );
        book
    }

    /// Spread = 1 (Bid 100, Ask 101) with MidPrice peg levels active
    fn book_with_mid_price_pegs() -> OrderBook {
        let mut book = basic_book();
        book.add_pegged_order(
            SequenceNumber(10),
            OrderId(10),
            pegged(PegReference::Primary, 20, Side::Buy),
        );
        book.add_pegged_order(
            SequenceNumber(11),
            OrderId(11),
            pegged(PegReference::Primary, 25, Side::Sell),
        );
        book.add_pegged_order(
            SequenceNumber(12),
            OrderId(12),
            pegged(PegReference::MidPrice, 15, Side::Buy),
        );
        book.add_pegged_order(
            SequenceNumber(13),
            OrderId(13),
            pegged(PegReference::MidPrice, 10, Side::Sell),
        );
        book
    }

    // ==================== buy_sell_pressure ====================

    #[test]
    fn buy_sell_pressure_empty_book() {
        let (buy, sell) = empty_book().buy_sell_pressure();
        assert_eq!(buy, Quantity(0));
        assert_eq!(sell, Quantity(0));
    }

    #[test]
    fn buy_sell_pressure_limit_only() {
        let book = basic_book();
        let (buy, sell) = book.buy_sell_pressure();
        assert_eq!(buy, Quantity(80)); // 50 + 30
        assert_eq!(sell, Quantity(100)); // 40 + 60
    }

    #[test]
    fn buy_sell_pressure_with_pegs() {
        let book = book_with_pegs();
        let (buy, sell) = book.buy_sell_pressure();
        assert_eq!(buy, Quantity(100)); // 80 limit + 20 peg
        assert_eq!(sell, Quantity(125)); // 100 limit + 25 peg
    }

    #[test]
    fn buy_sell_pressure_includes_hidden() {
        let book = book_with_hidden();
        let (buy, sell) = book.buy_sell_pressure();
        // Bid: (50+10) + (30+20) = 110
        assert_eq!(buy, Quantity(110));
        // Ask: (40+5) + (60+15) = 120
        assert_eq!(sell, Quantity(120));
    }

    #[test]
    fn buy_sell_pressure_includes_all_peg_references() {
        let book = book_with_mid_price_pegs();
        let (buy, sell) = book.buy_sell_pressure();
        // Bid: 80 limit + 20 primary + 15 midprice = 115
        assert_eq!(buy, Quantity(115));
        // Ask: 100 limit + 25 primary + 10 midprice = 135
        assert_eq!(sell, Quantity(135));
    }

    // ==================== price_at_depth ====================

    #[test]
    fn price_at_depth_empty_book() {
        assert!(
            empty_book()
                .price_at_depth(Side::Buy, Quantity(1))
                .is_none()
        );
        assert!(
            empty_book()
                .price_at_depth(Side::Sell, Quantity(1))
                .is_none()
        );
    }

    #[test]
    fn price_at_depth_first_level_buy() {
        let book = basic_book();
        // Bid: 100(50), 99(30). Depth 50 => price 100
        assert_eq!(
            book.price_at_depth(Side::Buy, Quantity(50)),
            Some(Price(100))
        );
    }

    #[test]
    fn price_at_depth_first_level_sell() {
        let book = basic_book();
        // Ask: 101(40), 102(60). Depth 40 => price 101
        assert_eq!(
            book.price_at_depth(Side::Sell, Quantity(40)),
            Some(Price(101))
        );
    }

    #[test]
    fn price_at_depth_spans_multiple_levels_buy() {
        let book = basic_book();
        // Bid: 100(50) + 99(30). Depth 60 => price 99
        assert_eq!(
            book.price_at_depth(Side::Buy, Quantity(60)),
            Some(Price(99))
        );
    }

    #[test]
    fn price_at_depth_spans_multiple_levels_sell() {
        let book = basic_book();
        // Ask: 101(40) + 102(60). Depth 90 => price 102
        assert_eq!(
            book.price_at_depth(Side::Sell, Quantity(90)),
            Some(Price(102))
        );
    }

    #[test]
    fn price_at_depth_exceeds_total_returns_none() {
        let book = basic_book();
        assert!(book.price_at_depth(Side::Buy, Quantity(81)).is_none());
        assert!(book.price_at_depth(Side::Sell, Quantity(101)).is_none());
    }

    #[test]
    fn price_at_depth_with_primary_peg() {
        let book = book_with_pegs();
        // Buy side: primary peg 20 + bid 100(50) + bid 99(30)
        assert_eq!(
            book.price_at_depth(Side::Buy, Quantity(20)),
            Some(Price(100))
        );

        // Depth 70 => 20 peg + 50 at 100 = 70 => price 100
        assert_eq!(
            book.price_at_depth(Side::Buy, Quantity(70)),
            Some(Price(100))
        );

        // Depth 71 => needs level 99 => price 99
        assert_eq!(
            book.price_at_depth(Side::Buy, Quantity(71)),
            Some(Price(99))
        );
    }

    #[test]
    fn price_at_depth_with_mid_price_peg_active() {
        let book = book_with_mid_price_pegs();
        // Spread = 101 - 100 = 1, so MidPrice peg IS active
        // Buy side: midprice peg 15 + primary peg 20 + bid 100(50) + bid 99(30)
        // Depth 35 => 15 mid + 20 primary = 35, need first limit level => price 100
        assert_eq!(
            book.price_at_depth(Side::Buy, Quantity(35)),
            Some(Price(100))
        );

        // Sell side: midprice peg 10 + primary peg 25 + ask 101(40) + ask 102(60)
        // Depth 75 => 10 mid + 25 primary + 40 at 101 = 75 => price 101
        assert_eq!(
            book.price_at_depth(Side::Sell, Quantity(75)),
            Some(Price(101))
        );
    }

    #[test]
    fn price_at_depth_mid_price_peg_inactive_wide_spread() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(SequenceNumber(1), OrderId(1), standard(100, 50, Side::Buy));
        book.add_limit_order(SequenceNumber(2), OrderId(2), standard(102, 40, Side::Sell)); // spread = 2 > 1
        book.add_pegged_order(
            SequenceNumber(10),
            OrderId(10),
            pegged(PegReference::MidPrice, 100, Side::Buy),
        );

        // Spread = 2 => MidPrice NOT active, only Primary (which is 0 here)
        // Buy: Depth 50 => price 100
        assert_eq!(
            book.price_at_depth(Side::Buy, Quantity(50)),
            Some(Price(100))
        );
        // Depth 51 => not enough => None
        assert!(book.price_at_depth(Side::Buy, Quantity(51)).is_none());
    }

    // ==================== vwap ====================

    #[test]
    fn vwap_zero_quantity() {
        let book = basic_book();
        assert!(book.vwap(Side::Buy, Quantity(0)).is_none());
    }

    #[test]
    fn vwap_empty_book() {
        assert!(empty_book().vwap(Side::Buy, Quantity(10)).is_none());
        assert!(empty_book().vwap(Side::Sell, Quantity(10)).is_none());
    }

    #[test]
    fn vwap_single_level_buy() {
        let book = basic_book();
        // Buying 40 from ask 101(40): cost = 40*101 = 4040, vwap = 4040/40 = 101.0
        let v = book.vwap(Side::Buy, Quantity(40)).unwrap();
        assert!((v - 101.0).abs() < EPS);
    }

    #[test]
    fn vwap_single_level_sell() {
        let book = basic_book();
        // Selling 50 into bid 100(50): cost = 50*100 = 5000, vwap = 5000/50 = 100.0
        let v = book.vwap(Side::Sell, Quantity(50)).unwrap();
        assert!((v - 100.0).abs() < EPS);
    }

    #[test]
    fn vwap_spans_multiple_levels_buy() {
        let book = basic_book();
        // Buying 70: 40 at 101 + 30 at 102 = 4040 + 3060 = 7100
        // vwap = 7100 / 70 ≈ 101.4286
        let v = book.vwap(Side::Buy, Quantity(70)).unwrap();
        assert!((v - 7100.0 / 70.0).abs() < EPS);
    }

    #[test]
    fn vwap_spans_multiple_levels_sell() {
        let book = basic_book();
        // Selling 70: 50 at 100 + 20 at 99 = 5000 + 1980 = 6980
        // vwap = 6980 / 70 ≈ 99.7143
        let v = book.vwap(Side::Sell, Quantity(70)).unwrap();
        assert!((v - 6980.0 / 70.0).abs() < EPS);
    }

    #[test]
    fn vwap_exceeds_liquidity() {
        let book = basic_book();
        // Total ask liquidity: 100. Requesting 101 => None
        assert!(book.vwap(Side::Buy, Quantity(101)).is_none());
        // Total bid liquidity: 80. Requesting 81 => None
        assert!(book.vwap(Side::Sell, Quantity(81)).is_none());
    }

    #[test]
    fn vwap_with_primary_peg_buy() {
        let book = book_with_pegs();
        // Buy side: primary ask peg 25 at best_ask(101), then ask levels 101(40), 102(60)
        // Buying 25: all from peg at 101 => vwap = 101.0
        let v = book.vwap(Side::Buy, Quantity(25)).unwrap();
        assert!((v - 101.0).abs() < EPS);

        // Buying 65: 25 peg at 101 + 40 at 101 = 65 at 101 => vwap = 101.0
        let v = book.vwap(Side::Buy, Quantity(65)).unwrap();
        assert!((v - 101.0).abs() < EPS);

        // Buying 80: 25 peg at 101 + 40 at 101 + 15 at 102
        // cost = 25*101 + 40*101 + 15*102 = 2525 + 4040 + 1530 = 8095
        let v = book.vwap(Side::Buy, Quantity(80)).unwrap();
        assert!((v - 8095.0 / 80.0).abs() < EPS);
    }

    #[test]
    fn vwap_with_primary_peg_sell() {
        let book = book_with_pegs();
        // Sell side: primary bid peg 20 at best_bid(100), then bid levels 100(50), 99(30)
        // Selling 20: all from peg at 100 => vwap = 100.0
        let v = book.vwap(Side::Sell, Quantity(20)).unwrap();
        assert!((v - 100.0).abs() < EPS);

        // Selling 70: 20 peg at 100 + 50 at 100 = 70 at 100 => vwap = 100.0
        let v = book.vwap(Side::Sell, Quantity(70)).unwrap();
        assert!((v - 100.0).abs() < EPS);
    }

    #[test]
    fn vwap_with_mid_price_peg_active() {
        let book = book_with_mid_price_pegs();
        // Spread = 1, mid peg active
        // Buy: midprice peg 10 at 101 + primary peg 25 at 101 + ask 101(40) + ask 102(60)
        // Buying 10: 10 mid peg at 101 => vwap = 101.0
        let v = book.vwap(Side::Buy, Quantity(10)).unwrap();
        assert!((v - 101.0).abs() < EPS);

        // Buying 35: 10 mid + 25 primary = 35 at 101 => vwap = 101.0
        let v = book.vwap(Side::Buy, Quantity(35)).unwrap();
        assert!((v - 101.0).abs() < EPS);
    }

    // ==================== market_impact ====================

    #[test]
    fn market_impact_zero_quantity() {
        let book = basic_book();
        let impact = book.market_impact(Side::Buy, Quantity(0));
        assert_eq!(impact.requested_quantity(), Quantity(0));
        assert_eq!(impact.available_quantity(), Quantity(0));
        assert_eq!(impact.consumed_price_levels(), 0);
    }

    #[test]
    fn market_impact_empty_book() {
        let impact = empty_book().market_impact(Side::Buy, Quantity(100));
        assert_eq!(impact.requested_quantity(), Quantity(100));
        assert_eq!(impact.available_quantity(), Quantity(0));
        assert_eq!(impact.best_price(), Price(0));
        assert_eq!(impact.worst_price(), Price(0));
        assert_eq!(impact.consumed_price_levels(), 0);
    }

    #[test]
    fn market_impact_single_level_buy() {
        let book = basic_book();
        let impact = book.market_impact(Side::Buy, Quantity(40));
        assert_eq!(impact.requested_quantity(), Quantity(40));
        assert_eq!(impact.available_quantity(), Quantity(40));
        assert_eq!(impact.best_price(), Price(101));
        assert_eq!(impact.worst_price(), Price(101));
        assert_eq!(impact.consumed_price_levels(), 1);
        assert_eq!(impact.slippage(), 0);
        assert!((impact.average_price() - 101.0).abs() < EPS);
    }

    #[test]
    fn market_impact_single_level_sell() {
        let book = basic_book();
        let impact = book.market_impact(Side::Sell, Quantity(50));
        assert_eq!(impact.requested_quantity(), Quantity(50));
        assert_eq!(impact.available_quantity(), Quantity(50));
        assert_eq!(impact.best_price(), Price(100));
        assert_eq!(impact.worst_price(), Price(100));
        assert_eq!(impact.consumed_price_levels(), 1);
        assert_eq!(impact.slippage(), 0);
    }

    #[test]
    fn market_impact_multi_level_buy() {
        let book = basic_book();
        let impact = book.market_impact(Side::Buy, Quantity(70));
        assert_eq!(impact.available_quantity(), Quantity(70));
        assert_eq!(impact.best_price(), Price(101));
        assert_eq!(impact.worst_price(), Price(102));
        assert_eq!(impact.consumed_price_levels(), 2);
        assert_eq!(impact.slippage(), 1);
        // cost: 40*101 + 30*102 = 4040 + 3060 = 7100
        assert!((impact.average_price() - 7100.0 / 70.0).abs() < EPS);
    }

    #[test]
    fn market_impact_multi_level_sell() {
        let book = basic_book();
        let impact = book.market_impact(Side::Sell, Quantity(70));
        assert_eq!(impact.available_quantity(), Quantity(70));
        assert_eq!(impact.best_price(), Price(100));
        assert_eq!(impact.worst_price(), Price(99));
        assert_eq!(impact.consumed_price_levels(), 2);
        assert_eq!(impact.slippage(), 1);
    }

    #[test]
    fn market_impact_partial_fill() {
        let book = basic_book();
        // Total ask: 100, requesting 150 => partial fill
        let impact = book.market_impact(Side::Buy, Quantity(150));
        assert_eq!(impact.requested_quantity(), Quantity(150));
        assert_eq!(impact.available_quantity(), Quantity(100));
        assert_eq!(impact.consumed_price_levels(), 2);
    }

    #[test]
    fn market_impact_with_primary_peg() {
        let book = book_with_pegs();
        // Buy: primary ask peg 25, then ask 101(40), 102(60)
        // Buying 25: filled by peg at best_ask 101
        let impact = book.market_impact(Side::Buy, Quantity(25));
        assert_eq!(impact.available_quantity(), Quantity(25));
        assert_eq!(impact.best_price(), Price(101));
        assert_eq!(impact.worst_price(), Price(101));
        assert_eq!(impact.consumed_price_levels(), 1);

        // Buying 65: 25 peg + 40 at 101 = 65
        let impact = book.market_impact(Side::Buy, Quantity(65));
        assert_eq!(impact.available_quantity(), Quantity(65));
        assert_eq!(impact.consumed_price_levels(), 1);
        assert_eq!(impact.worst_price(), Price(101));
    }

    #[test]
    fn market_impact_with_mid_price_peg_active() {
        let book = book_with_mid_price_pegs();
        // Spread = 1, mid peg active
        // Sell: midprice bid peg 15 + primary bid peg 20 + bid 100(50) + bid 99(30)
        let impact = book.market_impact(Side::Sell, Quantity(15));
        assert_eq!(impact.available_quantity(), Quantity(15));
        assert_eq!(impact.best_price(), Price(100));
        assert_eq!(impact.worst_price(), Price(100));
        assert_eq!(impact.consumed_price_levels(), 1);

        // Selling 85: 15 mid + 20 primary + 50 at 100 = 85 => consumed 1 limit level
        let impact = book.market_impact(Side::Sell, Quantity(85));
        assert_eq!(impact.available_quantity(), Quantity(85));
        assert_eq!(impact.consumed_price_levels(), 1);
        assert_eq!(impact.worst_price(), Price(100));
    }

    #[test]
    fn market_impact_slippage_across_levels() {
        let book = basic_book();
        // Buy all 100: 40 at 101 + 60 at 102
        let impact = book.market_impact(Side::Buy, Quantity(100));
        assert_eq!(impact.slippage(), 1); // 102 - 101
        // Sell all 80: 50 at 100 + 30 at 99
        let impact = book.market_impact(Side::Sell, Quantity(80));
        assert_eq!(impact.slippage(), 1); // 100 - 99
    }
}
