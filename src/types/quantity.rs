use std::{
    fmt,
    iter::Sum,
    ops::{Add, AddAssign, Sub, SubAssign},
};

/// Represents a quantity
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantity(pub u64);

impl Quantity {
    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    /// Check if the quantity is zero
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Convert the quantity to a f64
    pub fn as_f64(self) -> f64 {
        self.0 as f64
    }
}

impl Add for Quantity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Quantity {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Quantity {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Quantity {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Sum for Quantity {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(0), |acc, x| acc.saturating_add(x))
    }
}

impl<'a> Sum<&'a Quantity> for Quantity {
    fn sum<I: Iterator<Item = &'a Quantity>>(iter: I) -> Self {
        iter.fold(Self(0), |acc, x| acc.saturating_add(*x))
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturating_add() {
        assert_eq!(Quantity(100).saturating_add(Quantity(100)), Quantity(200));
        assert_eq!(
            Quantity(100).saturating_add(Quantity(1000000000000000000)),
            Quantity(1000000000000000100)
        );
    }

    #[test]
    fn test_saturating_sub() {
        assert_eq!(Quantity(100).saturating_sub(Quantity(100)), Quantity(0));
        assert_eq!(
            Quantity(100).saturating_sub(Quantity(1000000000000000000)),
            Quantity(0)
        );
        assert_eq!(Quantity(300).saturating_sub(Quantity(200)), Quantity(100));
    }

    #[test]
    fn test_is_zero() {
        assert!(Quantity(0).is_zero());
        assert!(!Quantity(1).is_zero());
        assert!(!Quantity(100).is_zero());
    }

    #[test]
    fn test_as_f64() {
        assert_eq!(Quantity(100).as_f64(), 100.0);
        assert_eq!(Quantity(1000).as_f64(), 1000.0);
        assert_eq!(Quantity(10000).as_f64(), 10000.0);
    }

    #[test]
    fn test_add() {
        assert_eq!(Quantity(100) + Quantity(100), Quantity(200));
        assert_eq!(
            Quantity(100) + Quantity(1000000000000000000),
            Quantity(1000000000000000100)
        );
    }

    #[test]
    fn test_add_assign() {
        let mut quantity = Quantity(100);
        quantity += Quantity(100);
        assert_eq!(quantity, Quantity(200));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Quantity(100) - Quantity(100), Quantity(0));
        assert_eq!(Quantity(300) - Quantity(200), Quantity(100));
    }

    #[test]
    fn test_sub_assign() {
        let mut quantity = Quantity(100);
        quantity -= Quantity(100);
        assert_eq!(quantity, Quantity(0));
    }

    #[test]
    fn test_sum() {
        assert_eq!(
            [Quantity(100), Quantity(100), Quantity(100)]
                .iter()
                .sum::<Quantity>(),
            Quantity(300)
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(Quantity(100).to_string(), "100");
        assert_eq!(
            Quantity(1000000000000000000).to_string(),
            "1000000000000000000"
        );
    }
}
