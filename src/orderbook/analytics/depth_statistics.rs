use crate::{Level2, LimitBook, Notional, Price, Quantity, Side};

use serde::{Deserialize, Serialize};

/// Represents the depth statistics of the order book
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepthStatistics {
    /// Number of price levels analyzed
    n_analyzed_levels: usize,

    /// Total value across all analyzed price levels
    total_value: Notional,

    /// Total size across all analyzed price levels
    total_size: Quantity,

    /// Average price level size
    average_level_size: f64,

    /// Smallest price level size
    min_level_size: Quantity,

    /// Largest price level size
    max_level_size: Quantity,

    /// Standard deviation of price level sizes
    std_dev_level_size: f64,
}

impl DepthStatistics {
    /// Compute the depth statistics of price levels (0 n_levels means all levels)
    pub(super) fn compute(book: &LimitBook, side: Side, n_levels: usize) -> Self {
        let mut stats = Self {
            n_analyzed_levels: 0,
            total_value: Notional(0),
            total_size: Quantity(0),
            average_level_size: 0.0,
            min_level_size: Quantity(u64::MAX),
            max_level_size: Quantity(0),
            std_dev_level_size: 0.0,
        };

        let n_levels = if n_levels == 0 { usize::MAX } else { n_levels };
        let mut sizes = Vec::new();

        match side {
            Side::Buy => {
                for (price, level) in book.bid_levels().iter().rev().take(n_levels) {
                    stats.observe_level(*price, level.total_quantity());
                    sizes.push(level.total_quantity());
                }
            }
            Side::Sell => {
                for (price, level) in book.ask_levels().iter().take(n_levels) {
                    stats.observe_level(*price, level.total_quantity());
                    sizes.push(level.total_quantity());
                }
            }
        }

        if stats.is_empty() {
            stats.min_level_size = Quantity(0);
            return stats;
        }

        stats.average_level_size = stats.total_size.as_f64() / stats.n_analyzed_levels as f64;

        let variance = sizes
            .iter()
            .map(|size| (size.as_f64() - stats.average_level_size).powi(2))
            .sum::<f64>()
            / stats.n_analyzed_levels as f64;
        stats.std_dev_level_size = variance.sqrt();

        stats
    }

    /// Compute the depth statistics of price levels (0 n_levels means all levels)
    /// from a level 2 market data snapshot
    pub(crate) fn compute_from_level2(level2: &Level2, side: Side, n_levels: usize) -> Self {
        let mut stats = Self {
            n_analyzed_levels: 0,
            total_value: Notional(0),
            total_size: Quantity(0),
            average_level_size: 0.0,
            min_level_size: Quantity(u64::MAX),
            max_level_size: Quantity(0),
            std_dev_level_size: 0.0,
        };

        let n_levels = if n_levels == 0 { usize::MAX } else { n_levels };
        let mut sizes = Vec::new();

        match side {
            Side::Buy => {
                for (price, quantity) in level2.bid_levels().iter().take(n_levels) {
                    stats.observe_level(*price, *quantity);
                    sizes.push(*quantity);
                }
            }
            Side::Sell => {
                for (price, quantity) in level2.ask_levels().iter().take(n_levels) {
                    stats.observe_level(*price, *quantity);
                    sizes.push(*quantity);
                }
            }
        }

        if stats.is_empty() {
            stats.min_level_size = Quantity(0);
            return stats;
        }

        stats.average_level_size = stats.total_size.as_f64() / stats.n_analyzed_levels as f64;

        let variance = sizes
            .iter()
            .map(|size| (size.as_f64() - stats.average_level_size).powi(2))
            .sum::<f64>()
            / stats.n_analyzed_levels as f64;
        stats.std_dev_level_size = variance.sqrt();

        stats
    }

    /// Observe a price level and update the statistics
    fn observe_level(&mut self, price: Price, quantity: Quantity) {
        self.n_analyzed_levels += 1;
        self.total_value = self.total_value.saturating_add(price * quantity);
        self.total_size = self.total_size.saturating_add(quantity);
        self.min_level_size = self.min_level_size.min(quantity);
        self.max_level_size = self.max_level_size.max(quantity);
    }

    /// Check if the statistics are empty
    pub fn is_empty(&self) -> bool {
        self.n_analyzed_levels == 0
    }

    /// Get the number of analyzed price levels
    pub fn n_analyzed_levels(&self) -> usize {
        self.n_analyzed_levels
    }

    /// Get the total value of all analyzed price levels
    pub fn total_value(&self) -> Notional {
        self.total_value
    }

    /// Get the total size of all analyzed price levels
    pub fn total_size(&self) -> Quantity {
        self.total_size
    }

    /// Get the average size of all analyzed price levels
    pub fn average_level_size(&self) -> f64 {
        self.average_level_size
    }

    /// Get the smallest size of all analyzed price levels
    pub fn min_level_size(&self) -> Quantity {
        self.min_level_size
    }

    /// Get the largest size of all analyzed price levels
    pub fn max_level_size(&self) -> Quantity {
        self.max_level_size
    }

    /// Get the standard deviation of the size of all analyzed price levels
    pub fn std_dev_level_size(&self) -> f64 {
        self.std_dev_level_size
    }

    /// Get the volume-weighted average price of all analyzed price levels
    pub fn vwap(&self) -> f64 {
        self.total_value / self.total_size
    }
}
