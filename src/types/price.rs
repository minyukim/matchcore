use std::{fmt, ops::Sub};

use serde::{Deserialize, Serialize};

/// Represents a price
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Price(pub u64);

impl Price {
    /// Check if the price is zero
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Calculate the absolute difference between two prices
    pub fn abs_diff(self, rhs: Self) -> u64 {
        self.0.abs_diff(rhs.0)
    }

    /// Convert the price to a f64
    pub fn as_f64(self) -> f64 {
        self.0 as f64
    }
}

impl Sub for Price {
    type Output = u64;

    /// Subtract two prices and return the difference
    /// It is intended to be used for calculating the spread of the order book
    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_zero() {
        assert!(Price(0).is_zero());
        assert!(!Price(1).is_zero());
        assert!(!Price(100).is_zero());
    }

    #[test]
    fn test_abs_diff() {
        assert_eq!(Price(100).abs_diff(Price(100)), 0);
        assert_eq!(Price(100).abs_diff(Price(101)), 1);
        assert_eq!(Price(100).abs_diff(Price(99)), 1);
    }

    #[test]
    fn test_as_f64() {
        assert_eq!(Price(100).as_f64(), 100.0);
        assert_eq!(Price(1000).as_f64(), 1000.0);
        assert_eq!(Price(10000).as_f64(), 10000.0);
    }

    #[test]
    fn test_display() {
        assert_eq!(Price(100).to_string(), "100");
        assert_eq!(Price(1000).to_string(), "1000");
        assert_eq!(Price(10000).to_string(), "10000");
    }
}
