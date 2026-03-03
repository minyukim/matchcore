use super::{Price, Quantity};

use std::{
    fmt,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
};

use serde::{Deserialize, Serialize};

/// Represents a notional value (price * quantity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Notional(pub u128);

impl Add for Notional {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Notional {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Notional {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Notional {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul<Quantity> for Price {
    type Output = Notional;

    fn mul(self, rhs: Quantity) -> Self::Output {
        Notional(self.0 as u128 * rhs.0 as u128)
    }
}

impl Mul<Price> for Quantity {
    type Output = Notional;

    fn mul(self, rhs: Price) -> Self::Output {
        Notional(self.0 as u128 * rhs.0 as u128)
    }
}

impl fmt::Display for Notional {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(Notional(100) + Notional(100), Notional(200));
        assert_eq!(
            Notional(100) + Notional(1000000000000000000),
            Notional(1000000000000000100)
        );
    }

    #[test]
    fn test_add_assign() {
        let mut notional = Notional(100);
        notional += Notional(100);
        assert_eq!(notional, Notional(200));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Notional(100) - Notional(100), Notional(0));
        assert_eq!(Notional(300) - Notional(200), Notional(100));
    }

    #[test]
    fn test_sub_assign() {
        let mut notional = Notional(100);
        notional -= Notional(100);
        assert_eq!(notional, Notional(0));
    }

    #[test]
    fn test_mul_price_quantity() {
        assert_eq!(Price(100) * Quantity(100), Notional(10000));
        assert_eq!(
            Price(100) * Quantity(1000000000000000000),
            Notional(100000000000000000000)
        );
    }

    #[test]
    fn test_mul_quantity_price() {
        assert_eq!(Quantity(100) * Price(100), Notional(10000));
        assert_eq!(
            Quantity(100) * Price(1000000000000000000),
            Notional(100000000000000000000)
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(Notional(100).to_string(), "100");
        assert_eq!(
            Notional(1000000000000000000).to_string(),
            "1000000000000000000"
        );
    }
}
