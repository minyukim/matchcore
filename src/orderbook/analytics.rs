mod depth_statistics;
mod market_impact;

pub use depth_statistics::*;
pub use market_impact::*;

use super::OrderBook;
use crate::{Notional, PegReference, Price, Quantity, Side};

impl OrderBook {
    /// Calculate the micro price, which weights the best bid and ask by the opposite side's liquidity
    ///
    /// Included quantities: visible and hidden
    pub fn micro_price(&self) -> Option<f64> {
        let (best_bid_price, best_bid_level) = self.limit.bid_levels.iter().next_back()?;
        let (best_ask_price, best_ask_level) = self.limit.ask_levels.iter().next()?;

        // Get volumes at best levels
        let bid_volume = best_bid_level.total_quantity();
        let ask_volume = best_ask_level.total_quantity();

        let total_volume = bid_volume.saturating_add(ask_volume);

        if total_volume.is_zero() {
            return None;
        }

        // micro_price = (ask_price * bid_volume + bid_price * ask_volume) / (bid_volume + ask_volume)
        let numerator = (*best_ask_price * bid_volume) + (*best_bid_price * ask_volume);
        let denominator = total_volume;

        Some(numerator / denominator)
    }

    /// Get the bid volume for the first N price levels
    pub fn bid_volume(&self, n_levels: usize) -> Quantity {
        self.limit
            .bid_levels
            .values()
            .rev()
            .take(n_levels)
            .map(|level| level.total_quantity())
            .sum::<Quantity>()
    }

    /// Get the ask volume for the first N price levels
    pub fn ask_volume(&self, n_levels: usize) -> Quantity {
        self.limit
            .ask_levels
            .values()
            .take(n_levels)
            .map(|level| level.total_quantity())
            .sum::<Quantity>()
    }

    /// Check if the order book is thin at the given threshold and number of levels
    ///
    /// Included quantities: visible and hidden
    pub fn is_thin_book(&self, threshold: Quantity, n_levels: usize) -> bool {
        let bid_volume = self.bid_volume(n_levels);
        let ask_volume = self.ask_volume(n_levels);

        bid_volume < threshold || ask_volume < threshold
    }

    /// Calculate the order book imbalance ratio for the top N levels
    ///
    /// Included quantities: visible and hidden
    pub fn order_book_imbalance(&self, n_levels: usize) -> f64 {
        let bid_volume = self.bid_volume(n_levels);
        let ask_volume = self.ask_volume(n_levels);

        let total_volume = bid_volume.saturating_add(ask_volume);

        if total_volume.is_zero() {
            return 0.0;
        }

        (bid_volume.as_f64() - ask_volume.as_f64()) / total_volume.as_f64()
    }

    /// Compute the depth statistics of price levels (0 n_levels means all levels)
    ///
    /// Included quantities: visible and hidden
    pub fn depth_statistics(&self, side: Side, n_levels: usize) -> DepthStatistics {
        DepthStatistics::compute(self, side, n_levels)
    }

    /// Compute the buy and sell pressure of the order book
    ///
    /// Included quantities: visible, hidden, and peg levels
    pub fn buy_sell_pressure(&self) -> (Quantity, Quantity) {
        let buy_limit_pressure = self.bid_volume(usize::MAX);
        let sell_limit_pressure = self.ask_volume(usize::MAX);
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
    ///
    /// Included quantities: visible, hidden, and active peg levels
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
                for (price, level) in self.limit.bid_levels.iter().rev() {
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
                for (price, level) in self.limit.ask_levels.iter() {
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
    ///
    /// Included quantities: visible, hidden, and active peg levels
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
                for (price, level) in self.limit.ask_levels.iter() {
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
                for (price, level) in self.limit.bid_levels.iter().rev() {
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
    ///
    /// Included quantities: visible, hidden, and active peg levels
    pub fn market_impact(&self, taker_side: Side, quantity: Quantity) -> MarketImpact {
        MarketImpact::compute(self, taker_side, quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LimitOrder, OrderFlags, OrderId, PegReference, PeggedOrder, Price, Quantity,
        QuantityPolicy, Side, TimeInForce,
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
        book.add_limit_order(OrderId(0), standard(100, 50, Side::Buy));
        book.add_limit_order(OrderId(1), standard(99, 30, Side::Buy));
        book.add_limit_order(OrderId(2), standard(101, 40, Side::Sell));
        book.add_limit_order(OrderId(3), standard(102, 60, Side::Sell));
        book
    }

    /// Same as basic_book but with hidden quantities:
    /// Bid 100 (vis 50, hid 10), Bid 99 (vis 30, hid 20),
    /// Ask 101 (vis 40, hid 5), Ask 102 (vis 60, hid 15)
    fn book_with_hidden() -> OrderBook {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(OrderId(0), iceberg(100, 50, 10, Side::Buy));
        book.add_limit_order(OrderId(1), iceberg(99, 30, 20, Side::Buy));
        book.add_limit_order(OrderId(2), iceberg(101, 40, 5, Side::Sell));
        book.add_limit_order(OrderId(3), iceberg(102, 60, 15, Side::Sell));
        book
    }

    /// basic_book + Primary peg bid 20, Primary peg ask 25
    fn book_with_pegs() -> OrderBook {
        let mut book = basic_book();
        book.add_pegged_order(OrderId(10), pegged(PegReference::Primary, 20, Side::Buy));
        book.add_pegged_order(OrderId(11), pegged(PegReference::Primary, 25, Side::Sell));
        book
    }

    /// Spread = 1 (Bid 100, Ask 101) with MidPrice peg levels active
    fn book_with_mid_price_pegs() -> OrderBook {
        let mut book = basic_book();
        book.add_pegged_order(OrderId(10), pegged(PegReference::Primary, 20, Side::Buy));
        book.add_pegged_order(OrderId(11), pegged(PegReference::Primary, 25, Side::Sell));
        book.add_pegged_order(OrderId(12), pegged(PegReference::MidPrice, 15, Side::Buy));
        book.add_pegged_order(OrderId(13), pegged(PegReference::MidPrice, 10, Side::Sell));
        book
    }

    // ==================== micro_price ====================

    #[test]
    fn micro_price_empty_book() {
        assert!(empty_book().micro_price().is_none());
    }

    #[test]
    fn micro_price_balanced_volumes() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(OrderId(0), standard(100, 100, Side::Buy));
        book.add_limit_order(OrderId(1), standard(102, 100, Side::Sell));

        // Equal volumes => micro_price = midpoint = (100 + 102) / 2 = 101
        let mp = book.micro_price().unwrap();
        assert!((mp - 101.0).abs() < EPS);
    }

    #[test]
    fn micro_price_imbalanced_toward_bid() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(OrderId(0), standard(100, 300, Side::Buy));
        book.add_limit_order(OrderId(1), standard(102, 100, Side::Sell));

        // micro = (102 * 300 + 100 * 100) / 400 = (30600 + 10000) / 400 = 101.5
        let mp = book.micro_price().unwrap();
        assert!((mp - 101.5).abs() < EPS);
    }

    #[test]
    fn micro_price_imbalanced_toward_ask() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(OrderId(0), standard(100, 100, Side::Buy));
        book.add_limit_order(OrderId(1), standard(102, 300, Side::Sell));

        // micro = (102 * 100 + 100 * 300) / 400 = (10200 + 30000) / 400 = 100.5
        let mp = book.micro_price().unwrap();
        assert!((mp - 100.5).abs() < EPS);
    }

    #[test]
    fn micro_price_includes_hidden_quantity() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(OrderId(0), iceberg(100, 50, 50, Side::Buy)); // total 100
        book.add_limit_order(OrderId(1), iceberg(102, 50, 50, Side::Sell)); // total 100

        let mp = book.micro_price().unwrap();
        assert!((mp - 101.0).abs() < EPS);
    }

    // ==================== bid_volume and ask_volume ====================

    #[test]
    fn bid_ask_volumes_empty_book() {
        let book = empty_book();
        assert_eq!(book.bid_volume(5), Quantity(0));
        assert_eq!(book.ask_volume(5), Quantity(0));
    }

    #[test]
    fn bid_ask_volumes_n_levels_zero() {
        let book = basic_book();
        assert_eq!(book.bid_volume(0), Quantity(0));
        assert_eq!(book.ask_volume(0), Quantity(0));
    }

    #[test]
    fn bid_ask_volumes_single_level() {
        let book = basic_book();
        assert_eq!(book.bid_volume(1), Quantity(50));
        assert_eq!(book.ask_volume(1), Quantity(40));
    }

    #[test]
    fn bid_ask_volumes_multiple_levels() {
        let book = basic_book();
        assert_eq!(book.bid_volume(2), Quantity(80));
        assert_eq!(book.ask_volume(2), Quantity(100));
    }

    #[test]
    fn bid_ask_volumes_exceeding_available_levels() {
        let book = basic_book();
        assert_eq!(book.bid_volume(100), Quantity(80));
        assert_eq!(book.ask_volume(100), Quantity(100));
    }

    #[test]
    fn bid_ask_volumes_includes_hidden() {
        let book = book_with_hidden();
        // Bid 100: 50+10=60, Bid 99: 30+20=50 => total = 110
        assert_eq!(book.bid_volume(2), Quantity(110));
        // Ask 101: 40+5=45, Ask 102: 60+15=75 => total = 120
        assert_eq!(book.ask_volume(2), Quantity(120));
    }

    // ==================== is_thin_book ====================

    #[test]
    fn is_thin_book_empty() {
        assert!(empty_book().is_thin_book(Quantity(1), 1));
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
        assert!(!empty_book().is_thin_book(Quantity(0), 1));
    }

    // ==================== order_book_imbalance ====================

    #[test]
    fn order_book_imbalance_empty_book() {
        assert_eq!(empty_book().order_book_imbalance(5), 0.0);
    }

    #[test]
    fn order_book_imbalance_balanced() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(OrderId(0), standard(100, 100, Side::Buy));
        book.add_limit_order(OrderId(1), standard(101, 100, Side::Sell));

        assert!((book.order_book_imbalance(1) - 0.0).abs() < EPS);
    }

    #[test]
    fn order_book_imbalance_all_bids() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(OrderId(0), standard(100, 100, Side::Buy));

        assert!((book.order_book_imbalance(1) - 1.0).abs() < EPS);
    }

    #[test]
    fn order_book_imbalance_all_asks() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(OrderId(0), standard(101, 100, Side::Sell));

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
        let stats = empty_book().depth_statistics(Side::Buy, 5);
        assert!(stats.is_empty());
        assert_eq!(stats.n_analyzed_levels(), 0);
        assert_eq!(stats.total_volume(), Quantity(0));
        assert_eq!(stats.min_level_size(), Quantity(0));
        assert_eq!(stats.max_level_size(), Quantity(0));
    }

    #[test]
    fn depth_statistics_single_level() {
        let mut book = OrderBook::new("TEST");
        book.add_limit_order(OrderId(0), standard(50, 100, Side::Buy));

        let stats = book.depth_statistics(Side::Buy, 1);
        assert_eq!(stats.n_analyzed_levels(), 1);
        assert_eq!(stats.total_volume(), Quantity(100));
        assert_eq!(stats.min_level_size(), Quantity(100));
        assert_eq!(stats.max_level_size(), Quantity(100));
        assert!((stats.average_level_size() - 100.0).abs() < EPS);
        assert!((stats.std_dev_level_size() - 0.0).abs() < EPS);
    }

    #[test]
    fn depth_statistics_multiple_levels() {
        let book = basic_book();
        let stats = book.depth_statistics(Side::Buy, 2);
        assert_eq!(stats.n_analyzed_levels(), 2);
        assert_eq!(stats.total_volume(), Quantity(80)); // 50 + 30
        assert_eq!(stats.min_level_size(), Quantity(30));
        assert_eq!(stats.max_level_size(), Quantity(50));
        assert!((stats.average_level_size() - 40.0).abs() < EPS);
    }

    #[test]
    fn depth_statistics_sell_side() {
        let book = basic_book();
        let stats = book.depth_statistics(Side::Sell, 2);
        assert_eq!(stats.n_analyzed_levels(), 2);
        assert_eq!(stats.total_volume(), Quantity(100)); // 40 + 60
        assert_eq!(stats.min_level_size(), Quantity(40));
        assert_eq!(stats.max_level_size(), Quantity(60));
        assert!((stats.average_level_size() - 50.0).abs() < EPS);
    }

    #[test]
    fn depth_statistics_zero_levels_means_all() {
        let book = basic_book();
        let stats = book.depth_statistics(Side::Buy, 0);
        assert_eq!(stats.n_analyzed_levels(), 2);
        assert_eq!(stats.total_volume(), Quantity(80));
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
        book.add_limit_order(OrderId(1), standard(100, 50, Side::Buy));
        book.add_limit_order(OrderId(2), standard(102, 40, Side::Sell)); // spread = 2 > 1
        book.add_pegged_order(OrderId(10), pegged(PegReference::MidPrice, 100, Side::Buy));

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
