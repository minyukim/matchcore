use crate::{DepthStatistics, MarketImpact, Notional, OrderBook, Price, Quantity, Side};

use std::fmt;

/// Represents the level 1 market data of the order book
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level2 {
    /// Bid side price levels, stored in a sorted vector from the best bid to the worst bid
    bid_levels: Vec<(Price, Quantity)>,
    /// Ask side price levels, stored in a sorted vector from the best ask to the worst ask
    ask_levels: Vec<(Price, Quantity)>,
}

impl From<&OrderBook> for Level2 {
    fn from(book: &OrderBook) -> Self {
        Self {
            bid_levels: book
                .limit
                .bids
                .iter()
                .rev()
                .map(|(price, level_id)| (*price, book.limit.levels[*level_id].total_quantity()))
                .collect(),
            ask_levels: book
                .limit()
                .asks
                .iter()
                .map(|(price, level_id)| (*price, book.limit.levels[*level_id].total_quantity()))
                .collect(),
        }
    }
}

impl Level2 {
    /// Get the bid side price levels, stored in a sorted vector
    /// from the best bid (highest price) to the worst bid (lowest price)
    pub fn bid_levels(&self) -> &Vec<(Price, Quantity)> {
        &self.bid_levels
    }

    /// Get the ask side price levels, stored in a sorted vector
    /// from the best ask (lowest price) to the worst ask (highest price)
    pub fn ask_levels(&self) -> &Vec<(Price, Quantity)> {
        &self.ask_levels
    }

    /// Get the best bid price and size, if exists
    pub fn best_bid(&self) -> Option<(Price, Quantity)> {
        self.bid_levels().first().copied()
    }

    /// Get the best ask price and size, if exists
    pub fn best_ask(&self) -> Option<(Price, Quantity)> {
        self.ask_levels().first().copied()
    }

    /// Get the best bid price, if exists
    pub fn best_bid_price(&self) -> Option<Price> {
        self.bid_levels().first().map(|(price, _)| *price)
    }

    /// Get the best ask price, if exists
    pub fn best_ask_price(&self) -> Option<Price> {
        self.ask_levels().first().map(|(price, _)| *price)
    }

    /// Get the best bid size, if exists
    pub fn best_bid_size(&self) -> Option<Quantity> {
        self.bid_levels().first().map(|(_, size)| *size)
    }

    /// Get the best ask size, if exists
    pub fn best_ask_size(&self) -> Option<Quantity> {
        self.ask_levels().first().map(|(_, size)| *size)
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
        self.bid_levels()
            .iter()
            .take(n_levels)
            .map(|(_, size)| *size)
            .sum::<Quantity>()
    }

    /// Get the ask size for the first N price levels
    pub fn ask_size(&self, n_levels: usize) -> Quantity {
        self.ask_levels()
            .iter()
            .take(n_levels)
            .map(|(_, size)| *size)
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
        DepthStatistics::compute_from_level2(self, side, n_levels)
    }

    /// Find the price where cumulative depth reaches the target quantity
    /// Return `None` if the target depth cannot be reached
    pub fn price_at_depth(&self, side: Side, depth: Quantity) -> Option<Price> {
        let mut cumulative = Quantity(0);

        match side {
            Side::Buy => {
                // Iterate over the limit bid price levels
                for (price, quantity) in self.bid_levels().iter() {
                    cumulative = cumulative.saturating_add(*quantity);
                    if cumulative >= depth {
                        return Some(*price);
                    }
                }

                None
            }
            Side::Sell => {
                // Iterate over the limit ask price levels
                for (price, quantity) in self.ask_levels().iter() {
                    cumulative = cumulative.saturating_add(*quantity);
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

        let mut remaining = quantity;
        let mut total_cost = Notional(0);
        let mut total_filled = Quantity(0);

        match taker_side {
            Side::Buy => {
                // Iterate over the limit ask price levels
                for (price, quantity) in self.ask_levels().iter() {
                    let fill_qty = remaining.min(*quantity);
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
                // Iterate over the limit bid price levels
                for (price, quantity) in self.bid_levels().iter() {
                    let fill_qty = remaining.min(*quantity);
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
        MarketImpact::compute_from_level2(self, taker_side, quantity)
    }
}

impl fmt::Display for Level2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (price, quantity) in self.ask_levels.iter().rev() {
            writeln!(f, "Ask: {} x {}", price, quantity)?;
        }
        for (price, quantity) in self.bid_levels.iter() {
            writeln!(f, "Bid: {} x {}", price, quantity)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LimitOrder, OrderFlags, OrderId, QuantityPolicy, SequenceNumber, TimeInForce};

    const EPS: f64 = 1e-9;

    fn standard(price: u64, qty: u64, side: Side) -> LimitOrder {
        LimitOrder::new(
            Price(price),
            QuantityPolicy::Standard {
                quantity: Quantity(qty),
            },
            OrderFlags::new(side, false, TimeInForce::Gtc),
        )
    }

    fn empty_l2() -> Level2 {
        Level2::from(&OrderBook::new("TEST"))
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

    fn basic_l2() -> Level2 {
        Level2::from(&basic_book())
    }

    // ==================== From<&OrderBook> ====================

    #[test]
    fn from_empty_orderbook() {
        let l2 = empty_l2();
        assert!(l2.bid_levels().is_empty());
        assert!(l2.ask_levels().is_empty());
    }

    #[test]
    fn from_basic_orderbook() {
        let l2 = basic_l2();
        assert_eq!(
            l2.bid_levels(),
            &vec![(Price(100), Quantity(50)), (Price(99), Quantity(30))]
        );
        assert_eq!(
            l2.ask_levels(),
            &vec![(Price(101), Quantity(40)), (Price(102), Quantity(60))]
        );
    }

    #[test]
    fn from_orderbook_bids_only() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(SequenceNumber(0), OrderId(0), standard(100, 10, Side::Buy));
        let l2 = Level2::from(&book);
        assert_eq!(l2.bid_levels().len(), 1);
        assert!(l2.ask_levels().is_empty());
    }

    #[test]
    fn from_orderbook_asks_only() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(SequenceNumber(0), OrderId(0), standard(101, 10, Side::Sell));
        let l2 = Level2::from(&book);
        assert!(l2.bid_levels().is_empty());
        assert_eq!(l2.ask_levels().len(), 1);
    }

    // ==================== best_bid / best_ask ====================

    #[test]
    fn best_bid_price_empty() {
        assert_eq!(empty_l2().best_bid_price(), None);
    }

    #[test]
    fn best_bid_price_populated() {
        assert_eq!(basic_l2().best_bid_price(), Some(Price(100)));
    }

    #[test]
    fn best_ask_price_empty() {
        assert_eq!(empty_l2().best_ask_price(), None);
    }

    #[test]
    fn best_ask_price_populated() {
        assert_eq!(basic_l2().best_ask_price(), Some(Price(101)));
    }

    #[test]
    fn best_bid_size_empty() {
        assert_eq!(empty_l2().best_bid_size(), None);
    }

    #[test]
    fn best_bid_size_populated() {
        assert_eq!(basic_l2().best_bid_size(), Some(Quantity(50)));
    }

    #[test]
    fn best_ask_size_empty() {
        assert_eq!(empty_l2().best_ask_size(), None);
    }

    #[test]
    fn best_ask_size_populated() {
        assert_eq!(basic_l2().best_ask_size(), Some(Quantity(40)));
    }

    // ==================== mid_price ====================

    #[test]
    fn mid_price_empty() {
        assert_eq!(empty_l2().mid_price(), None);
    }

    #[test]
    fn mid_price_one_side_only() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(SequenceNumber(0), OrderId(0), standard(100, 10, Side::Buy));
        assert_eq!(Level2::from(&book).mid_price(), None);
    }

    #[test]
    fn mid_price_populated() {
        let mid = basic_l2().mid_price().unwrap();
        assert!((mid - 100.5).abs() < EPS);
    }

    // ==================== spread ====================

    #[test]
    fn spread_empty() {
        assert_eq!(empty_l2().spread(), None);
    }

    #[test]
    fn spread_one_side_only() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(SequenceNumber(0), OrderId(0), standard(101, 10, Side::Sell));
        assert_eq!(Level2::from(&book).spread(), None);
    }

    #[test]
    fn spread_populated() {
        assert_eq!(basic_l2().spread(), Some(1));
    }

    // ==================== micro_price ====================

    #[test]
    fn micro_price_empty() {
        assert_eq!(empty_l2().micro_price(), None);
    }

    #[test]
    fn micro_price_equal_sizes() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(SequenceNumber(0), OrderId(0), standard(100, 10, Side::Buy));
        book.add_limit_order(SequenceNumber(1), OrderId(1), standard(102, 10, Side::Sell));
        let l2 = Level2::from(&book);
        let micro = l2.micro_price().unwrap();
        let mid = l2.mid_price().unwrap();
        assert!((micro - mid).abs() < EPS);
    }

    #[test]
    fn micro_price_skewed_toward_ask() {
        let l2 = basic_l2();
        let micro = l2.micro_price().unwrap();
        let mid = l2.mid_price().unwrap();
        assert!(micro > mid);
    }

    #[test]
    fn micro_price_skewed_toward_bid() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(SequenceNumber(0), OrderId(0), standard(100, 10, Side::Buy));
        book.add_limit_order(SequenceNumber(1), OrderId(1), standard(102, 90, Side::Sell));
        let l2 = Level2::from(&book);
        let micro = l2.micro_price().unwrap();
        let mid = l2.mid_price().unwrap();
        assert!(micro < mid);
    }

    // ==================== bid_size / ask_size ====================

    #[test]
    fn bid_size_all_levels() {
        assert_eq!(basic_l2().bid_size(10), Quantity(80));
    }

    #[test]
    fn bid_size_top_one() {
        assert_eq!(basic_l2().bid_size(1), Quantity(50));
    }

    #[test]
    fn bid_size_zero_levels() {
        assert_eq!(basic_l2().bid_size(0), Quantity(0));
    }

    #[test]
    fn ask_size_all_levels() {
        assert_eq!(basic_l2().ask_size(10), Quantity(100));
    }

    #[test]
    fn ask_size_top_one() {
        assert_eq!(basic_l2().ask_size(1), Quantity(40));
    }

    #[test]
    fn ask_size_empty() {
        assert_eq!(empty_l2().ask_size(5), Quantity(0));
    }

    // ==================== is_thin_book ====================

    #[test]
    fn thin_book_below_threshold() {
        assert!(basic_l2().is_thin_book(Quantity(100), 2));
    }

    #[test]
    fn thin_book_above_threshold() {
        assert!(!basic_l2().is_thin_book(Quantity(50), 2));
    }

    #[test]
    fn thin_book_empty() {
        assert!(empty_l2().is_thin_book(Quantity(1), 1));
    }

    // ==================== order_book_imbalance ====================

    #[test]
    fn imbalance_empty() {
        assert!((empty_l2().order_book_imbalance(5) - 0.0).abs() < EPS);
    }

    #[test]
    fn imbalance_balanced() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(SequenceNumber(0), OrderId(0), standard(100, 50, Side::Buy));
        book.add_limit_order(SequenceNumber(1), OrderId(1), standard(101, 50, Side::Sell));
        let l2 = Level2::from(&book);
        assert!((l2.order_book_imbalance(5) - 0.0).abs() < EPS);
    }

    #[test]
    fn imbalance_bid_heavy() {
        let imb = basic_l2().order_book_imbalance(10);
        assert!(imb < 0.0);
        let expected = (80.0 - 100.0) / 180.0;
        assert!((imb - expected).abs() < EPS);
    }

    #[test]
    fn imbalance_top_level_only() {
        let imb = basic_l2().order_book_imbalance(1);
        assert!(imb > 0.0);
        let expected = (50.0 - 40.0) / 90.0;
        assert!((imb - expected).abs() < EPS);
    }

    // ==================== depth_statistics ====================

    #[test]
    fn depth_statistics_bid_side() {
        let stats = basic_l2().depth_statistics(Side::Buy, 0);
        assert_eq!(stats.n_analyzed_levels(), 2);
        assert_eq!(stats.total_size(), Quantity(80));
        assert_eq!(stats.min_level_size(), Quantity(30));
        assert_eq!(stats.max_level_size(), Quantity(50));
    }

    #[test]
    fn depth_statistics_ask_side() {
        let stats = basic_l2().depth_statistics(Side::Sell, 0);
        assert_eq!(stats.n_analyzed_levels(), 2);
        assert_eq!(stats.total_size(), Quantity(100));
        assert_eq!(stats.min_level_size(), Quantity(40));
        assert_eq!(stats.max_level_size(), Quantity(60));
    }

    #[test]
    fn depth_statistics_limited_levels() {
        let stats = basic_l2().depth_statistics(Side::Buy, 1);
        assert_eq!(stats.n_analyzed_levels(), 1);
        assert_eq!(stats.total_size(), Quantity(50));
    }

    #[test]
    fn depth_statistics_empty() {
        let stats = empty_l2().depth_statistics(Side::Buy, 0);
        assert!(stats.is_empty());
        assert_eq!(stats.total_size(), Quantity(0));
    }

    // ==================== price_at_depth ====================

    #[test]
    fn price_at_depth_buy_first_level() {
        assert_eq!(
            basic_l2().price_at_depth(Side::Buy, Quantity(50)),
            Some(Price(100))
        );
    }

    #[test]
    fn price_at_depth_buy_second_level() {
        assert_eq!(
            basic_l2().price_at_depth(Side::Buy, Quantity(51)),
            Some(Price(99))
        );
    }

    #[test]
    fn price_at_depth_buy_exact_total() {
        assert_eq!(
            basic_l2().price_at_depth(Side::Buy, Quantity(80)),
            Some(Price(99))
        );
    }

    #[test]
    fn price_at_depth_buy_exceeds_total() {
        assert_eq!(basic_l2().price_at_depth(Side::Buy, Quantity(81)), None);
    }

    #[test]
    fn price_at_depth_sell_first_level() {
        assert_eq!(
            basic_l2().price_at_depth(Side::Sell, Quantity(40)),
            Some(Price(101))
        );
    }

    #[test]
    fn price_at_depth_sell_second_level() {
        assert_eq!(
            basic_l2().price_at_depth(Side::Sell, Quantity(41)),
            Some(Price(102))
        );
    }

    #[test]
    fn price_at_depth_sell_exceeds_total() {
        assert_eq!(basic_l2().price_at_depth(Side::Sell, Quantity(101)), None);
    }

    #[test]
    fn price_at_depth_empty() {
        assert_eq!(empty_l2().price_at_depth(Side::Buy, Quantity(1)), None);
    }

    // ==================== vwap ====================

    #[test]
    fn vwap_zero_quantity() {
        assert_eq!(basic_l2().vwap(Side::Buy, Quantity(0)), None);
    }

    #[test]
    fn vwap_buy_single_level() {
        let vwap = basic_l2().vwap(Side::Buy, Quantity(10)).unwrap();
        assert!((vwap - 101.0).abs() < EPS);
    }

    #[test]
    fn vwap_buy_exact_first_level() {
        let vwap = basic_l2().vwap(Side::Buy, Quantity(40)).unwrap();
        assert!((vwap - 101.0).abs() < EPS);
    }

    #[test]
    fn vwap_buy_spans_two_levels() {
        let vwap = basic_l2().vwap(Side::Buy, Quantity(50)).unwrap();
        // 40 @ 101 + 10 @ 102 = 4040 + 1020 = 5060 / 50
        assert!((vwap - 101.2).abs() < EPS);
    }

    #[test]
    fn vwap_buy_all_liquidity() {
        let vwap = basic_l2().vwap(Side::Buy, Quantity(100)).unwrap();
        // 40 @ 101 + 60 @ 102 = 4040 + 6120 = 10160 / 100
        assert!((vwap - 101.6).abs() < EPS);
    }

    #[test]
    fn vwap_buy_exceeds_liquidity() {
        assert_eq!(basic_l2().vwap(Side::Buy, Quantity(101)), None);
    }

    #[test]
    fn vwap_sell_single_level() {
        let vwap = basic_l2().vwap(Side::Sell, Quantity(10)).unwrap();
        assert!((vwap - 100.0).abs() < EPS);
    }

    #[test]
    fn vwap_sell_spans_two_levels() {
        let vwap = basic_l2().vwap(Side::Sell, Quantity(60)).unwrap();
        // 50 @ 100 + 10 @ 99 = 5000 + 990 = 5990 / 60
        let expected = 5990.0 / 60.0;
        assert!((vwap - expected).abs() < EPS);
    }

    #[test]
    fn vwap_sell_exceeds_liquidity() {
        assert_eq!(basic_l2().vwap(Side::Sell, Quantity(81)), None);
    }

    #[test]
    fn vwap_empty() {
        assert_eq!(empty_l2().vwap(Side::Buy, Quantity(1)), None);
    }

    // ==================== market_impact ====================

    #[test]
    fn market_impact_zero_quantity() {
        let impact = basic_l2().market_impact(Side::Buy, Quantity(0));
        assert_eq!(impact.requested_quantity(), Quantity(0));
        assert_eq!(impact.available_quantity(), Quantity(0));
        assert_eq!(impact.consumed_price_levels(), 0);
        assert!((impact.average_price() - 0.0).abs() < EPS);
    }

    #[test]
    fn market_impact_empty_book() {
        let impact = empty_l2().market_impact(Side::Buy, Quantity(10));
        assert_eq!(impact.requested_quantity(), Quantity(10));
        assert_eq!(impact.available_quantity(), Quantity(0));
        assert_eq!(impact.consumed_price_levels(), 0);
    }

    #[test]
    fn market_impact_buy_within_first_level() {
        let impact = basic_l2().market_impact(Side::Buy, Quantity(10));
        assert_eq!(impact.requested_quantity(), Quantity(10));
        assert_eq!(impact.available_quantity(), Quantity(10));
        assert_eq!(impact.total_cost(), Notional(10 * 101));
        assert_eq!(impact.worst_price(), Price(101));
        assert_eq!(impact.consumed_price_levels(), 1);
        assert_eq!(impact.slippage(), 0);
        assert!((impact.average_price() - 101.0).abs() < EPS);
    }

    #[test]
    fn market_impact_buy_exact_first_level() {
        let impact = basic_l2().market_impact(Side::Buy, Quantity(40));
        assert_eq!(impact.available_quantity(), Quantity(40));
        assert_eq!(impact.total_cost(), Notional(40 * 101));
        assert_eq!(impact.worst_price(), Price(101));
        assert_eq!(impact.consumed_price_levels(), 1);
        assert_eq!(impact.slippage(), 0);
    }

    #[test]
    fn market_impact_buy_spans_two_levels() {
        // 40 @ 101 + 10 @ 102
        let impact = basic_l2().market_impact(Side::Buy, Quantity(50));
        assert_eq!(impact.available_quantity(), Quantity(50));
        assert_eq!(impact.total_cost(), Notional(40 * 101 + 10 * 102));
        assert_eq!(impact.worst_price(), Price(102));
        assert_eq!(impact.consumed_price_levels(), 2);
        assert_eq!(impact.slippage(), 1);
        assert!((impact.average_price() - 101.2).abs() < EPS);
    }

    #[test]
    fn market_impact_buy_all_liquidity() {
        // 40 @ 101 + 60 @ 102
        let impact = basic_l2().market_impact(Side::Buy, Quantity(100));
        assert_eq!(impact.available_quantity(), Quantity(100));
        assert_eq!(impact.total_cost(), Notional(40 * 101 + 60 * 102));
        assert_eq!(impact.worst_price(), Price(102));
        assert_eq!(impact.consumed_price_levels(), 2);
        assert_eq!(impact.slippage(), 1);
    }

    #[test]
    fn market_impact_buy_exceeds_liquidity() {
        let impact = basic_l2().market_impact(Side::Buy, Quantity(150));
        assert_eq!(impact.requested_quantity(), Quantity(150));
        assert_eq!(impact.available_quantity(), Quantity(100));
        assert_eq!(impact.consumed_price_levels(), 2);
        assert_eq!(impact.slippage(), 1);
    }

    #[test]
    fn market_impact_sell_within_first_level() {
        let impact = basic_l2().market_impact(Side::Sell, Quantity(10));
        assert_eq!(impact.requested_quantity(), Quantity(10));
        assert_eq!(impact.available_quantity(), Quantity(10));
        assert_eq!(impact.total_cost(), Notional(10 * 100));
        assert_eq!(impact.worst_price(), Price(100));
        assert_eq!(impact.consumed_price_levels(), 1);
        assert_eq!(impact.slippage(), 0);
        assert!((impact.average_price() - 100.0).abs() < EPS);
    }

    #[test]
    fn market_impact_sell_spans_two_levels() {
        // 50 @ 100 + 10 @ 99
        let impact = basic_l2().market_impact(Side::Sell, Quantity(60));
        assert_eq!(impact.available_quantity(), Quantity(60));
        assert_eq!(impact.total_cost(), Notional(50 * 100 + 10 * 99));
        assert_eq!(impact.worst_price(), Price(99));
        assert_eq!(impact.consumed_price_levels(), 2);
        assert_eq!(impact.slippage(), 1);
        let expected_avg = (50.0 * 100.0 + 10.0 * 99.0) / 60.0;
        assert!((impact.average_price() - expected_avg).abs() < EPS);
    }

    #[test]
    fn market_impact_sell_exceeds_liquidity() {
        let impact = basic_l2().market_impact(Side::Sell, Quantity(100));
        assert_eq!(impact.requested_quantity(), Quantity(100));
        assert_eq!(impact.available_quantity(), Quantity(80));
        assert_eq!(impact.consumed_price_levels(), 2);
        assert_eq!(impact.slippage(), 1);
    }

    #[test]
    fn market_impact_sell_empty_book() {
        let impact = empty_l2().market_impact(Side::Sell, Quantity(10));
        assert_eq!(impact.available_quantity(), Quantity(0));
        assert_eq!(impact.consumed_price_levels(), 0);
    }

    // ==================== display ====================

    #[test]
    fn display_empty() {
        let l2 = empty_l2();
        println!("{l2}");
        assert_eq!(l2.to_string(), "");
    }

    #[test]
    fn display_populated() {
        let l2 = basic_l2();
        println!("{l2}");
        assert_eq!(
            l2.to_string(),
            "Ask: 102 x 60\nAsk: 101 x 40\nBid: 100 x 50\nBid: 99 x 30\n"
        );
    }
}
