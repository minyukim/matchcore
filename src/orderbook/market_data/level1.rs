use crate::{OrderBook, Price, Quantity};

use std::fmt;

/// Represents the level 1 market data of the order book
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level1 {
    /// The last trade price
    last_trade_price: Option<Price>,
    /// The best bid price and size
    best_bid: Option<(Price, Quantity)>,
    /// The best ask price and size
    best_ask: Option<(Price, Quantity)>,
}

impl From<&OrderBook> for Level1 {
    fn from(book: &OrderBook) -> Self {
        Self {
            last_trade_price: book.last_trade_price(),
            best_bid: book.best_bid(),
            best_ask: book.best_ask(),
        }
    }
}

impl Level1 {
    /// Get the last trade price, `None` if no trade has occurred yet
    pub fn last_trade_price(&self) -> Option<Price> {
        self.last_trade_price
    }

    /// Get the best bid price and size, if exists
    pub fn best_bid(&self) -> Option<(Price, Quantity)> {
        self.best_bid
    }

    /// Get the best ask price and size, if exists
    pub fn best_ask(&self) -> Option<(Price, Quantity)> {
        self.best_ask
    }

    /// Get the best bid price, if exists
    pub fn best_bid_price(&self) -> Option<Price> {
        self.best_bid.map(|(price, _)| price)
    }

    /// Get the best ask price, if exists
    pub fn best_ask_price(&self) -> Option<Price> {
        self.best_ask.map(|(price, _)| price)
    }

    /// Get the best bid size, if exists
    pub fn best_bid_size(&self) -> Option<Quantity> {
        self.best_bid.map(|(_, size)| size)
    }

    /// Get the best ask size, if exists
    pub fn best_ask_size(&self) -> Option<Quantity> {
        self.best_ask.map(|(_, size)| size)
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
}

impl fmt::Display for Level1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Last Trade Price: {}",
            self.last_trade_price()
                .map(|p| p.to_string())
                .unwrap_or("None".to_string())
        )?;

        writeln!(
            f,
            "Best Ask: {}",
            self.best_ask()
                .map(|(p, q)| format!("{} x {}", p, q))
                .unwrap_or("None".to_string())
        )?;

        writeln!(
            f,
            "Best Bid: {}",
            self.best_bid()
                .map(|(p, q)| format!("{} x {}", p, q))
                .unwrap_or("None".to_string())
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    const EPS: f64 = 1e-9;

    fn empty_level1() -> Level1 {
        Level1 {
            last_trade_price: None,
            best_bid: None,
            best_ask: None,
        }
    }

    fn populated_level1() -> Level1 {
        Level1 {
            last_trade_price: Some(Price(150)),
            best_bid: Some((Price(100), Quantity(500))),
            best_ask: Some((Price(200), Quantity(300))),
        }
    }

    #[test]
    fn test_from_orderbook() {
        let book = OrderBook::new("TEST");
        let l1 = Level1::from(&book);
        assert_eq!(l1.last_trade_price(), None);
        assert_eq!(l1.best_bid(), None);
        assert_eq!(l1.best_ask(), None);
        assert_eq!(l1.best_bid_price(), None);
        assert_eq!(l1.best_ask_price(), None);
        assert_eq!(l1.best_bid_size(), None);
        assert_eq!(l1.best_ask_size(), None);
    }

    #[test]
    fn test_getters_all_none() {
        let l1 = empty_level1();
        assert_eq!(l1.last_trade_price(), None);
        assert_eq!(l1.best_bid(), None);
        assert_eq!(l1.best_ask(), None);
        assert_eq!(l1.best_bid_price(), None);
        assert_eq!(l1.best_ask_price(), None);
        assert_eq!(l1.best_bid_size(), None);
        assert_eq!(l1.best_ask_size(), None);
    }

    #[test]
    fn test_getters_all_some() {
        let l1 = populated_level1();
        assert_eq!(l1.last_trade_price(), Some(Price(150)));
        assert_eq!(l1.best_bid(), Some((Price(100), Quantity(500))));
        assert_eq!(l1.best_ask(), Some((Price(200), Quantity(300))));
        assert_eq!(l1.best_bid_price(), Some(Price(100)));
        assert_eq!(l1.best_ask_price(), Some(Price(200)));
        assert_eq!(l1.best_bid_size(), Some(Quantity(500)));
        assert_eq!(l1.best_ask_size(), Some(Quantity(300)));
    }

    #[test]
    fn test_spread_with_both_sides() {
        let l1 = populated_level1();
        assert_eq!(l1.spread(), Some(100));
    }

    #[test]
    fn test_spread_missing_bid() {
        let l1 = Level1 {
            best_bid: None,
            best_ask: Some((Price(200), Quantity(300))),
            ..empty_level1()
        };
        assert_eq!(l1.spread(), None);
    }

    #[test]
    fn test_spread_missing_ask() {
        let l1 = Level1 {
            best_bid: Some((Price(100), Quantity(500))),
            best_ask: None,
            ..empty_level1()
        };
        assert_eq!(l1.spread(), None);
    }

    #[test]
    fn test_spread_missing_both() {
        let l1 = empty_level1();
        assert_eq!(l1.spread(), None);
    }

    #[test]
    fn test_spread_narrow() {
        let l1 = Level1 {
            best_bid: Some((Price(99), Quantity(500))),
            best_ask: Some((Price(100), Quantity(300))),
            ..empty_level1()
        };
        assert_eq!(l1.spread(), Some(1));
    }

    #[test]
    fn test_mid_price_with_both_sides() {
        let l1 = populated_level1();
        assert_eq!(l1.mid_price(), Some(150.0));
    }

    #[test]
    fn test_mid_price_missing_bid() {
        let l1 = Level1 {
            best_bid: None,
            best_ask: Some((Price(200), Quantity(300))),
            ..empty_level1()
        };
        assert_eq!(l1.mid_price(), None);
    }

    #[test]
    fn test_mid_price_missing_ask() {
        let l1 = Level1 {
            best_bid: Some((Price(100), Quantity(500))),
            best_ask: None,
            ..empty_level1()
        };
        assert_eq!(l1.mid_price(), None);
    }

    #[test]
    fn test_mid_price_missing_both() {
        let l1 = empty_level1();
        assert_eq!(l1.mid_price(), None);
    }

    #[test]
    fn test_micro_price_empty_book() {
        assert!(empty_level1().micro_price().is_none());
    }

    #[test]
    fn test_micro_price_balanced_sizes() {
        let l1 = Level1 {
            best_bid: Some((Price(100), Quantity(100))),
            best_ask: Some((Price(102), Quantity(100))),
            ..empty_level1()
        };

        // Equal sizes => micro_price = midpoint = (100 + 102) / 2 = 101
        let mp = l1.micro_price().unwrap();
        assert!((mp - 101.0).abs() < EPS);
    }

    #[test]
    fn test_micro_price_imbalanced_toward_bid() {
        let l1 = Level1 {
            best_bid: Some((Price(100), Quantity(300))),
            best_ask: Some((Price(102), Quantity(100))),
            ..empty_level1()
        };

        // micro = (102 * 300 + 100 * 100) / 400 = (30600 + 10000) / 400 = 101.5
        let mp = l1.micro_price().unwrap();
        assert!((mp - 101.5).abs() < EPS);
    }

    #[test]
    fn test_micro_price_imbalanced_toward_ask() {
        let l1 = Level1 {
            best_bid: Some((Price(100), Quantity(100))),
            best_ask: Some((Price(102), Quantity(300))),
            ..empty_level1()
        };

        // micro = (102 * 100 + 100 * 300) / 400 = (10200 + 30000) / 400 = 100.5
        let mp = l1.micro_price().unwrap();
        assert!((mp - 100.5).abs() < EPS);
    }

    #[test]
    fn test_display() {
        let l1 = empty_level1();
        println!("{}", l1);
        assert_eq!(
            l1.to_string(),
            "Last Trade Price: None\nBest Ask: None\nBest Bid: None\n"
        );

        let l1 = populated_level1();
        println!("{}", l1);
        assert_eq!(
            l1.to_string(),
            "Last Trade Price: 150\nBest Ask: 200 x 300\nBest Bid: 100 x 500\n"
        );
    }
}
