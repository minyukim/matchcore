use crate::{Side, report::Trade};

use serde::{Deserialize, Serialize};

/// Result of a match operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// The side of the taker order
    taker_side: Side,
    /// The total executed quantity during the match
    executed_quantity: u64,
    /// The total value of the trades made during the match
    executed_value: u64,
    /// The trades that were made during the match
    trades: Vec<Trade>,
}

impl MatchResult {
    /// Create a new match result
    pub fn new(taker_side: Side) -> Self {
        Self {
            taker_side,
            executed_quantity: 0,
            executed_value: 0,
            trades: Vec::new(),
        }
    }

    /// Get the side of the taker order
    pub fn taker_side(&self) -> Side {
        self.taker_side
    }

    /// Get the total executed quantity during the match
    pub fn executed_quantity(&self) -> u64 {
        self.executed_quantity
    }

    /// Get the total value of the trades made during the match
    pub fn executed_value(&self) -> u64 {
        self.executed_value
    }

    /// Get the trades that were made during the match
    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    /// Add a trade to the match result
    pub(crate) fn add_trade(&mut self, trade: Trade) {
        let price = trade.price();
        let quantity = trade.quantity();

        self.executed_quantity += quantity;
        self.executed_value += price * quantity;

        self.trades.push(trade);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Side, report::Trade};

    fn create_match_result() -> MatchResult {
        MatchResult::new(Side::Buy)
    }

    #[test]
    fn test_taker_side() {
        assert_eq!(create_match_result().taker_side(), Side::Buy);
    }

    #[test]
    fn test_executed_quantity() {
        assert_eq!(create_match_result().executed_quantity(), 0);
    }

    #[test]
    fn test_executed_value() {
        assert_eq!(create_match_result().executed_value(), 0);
    }

    #[test]
    fn test_trades() {
        let mut match_result = create_match_result();
        assert_eq!(match_result.trades(), &[]);

        let trades = [
            Trade::new(2, 99, 20),
            Trade::new(3, 100, 30),
            Trade::new(4, 101, 20),
        ];
        let expected_executed_quantities = [20, 50, 70];
        let expected_executed_values = [1980, 4980, 7000];

        for (i, trade) in trades.iter().enumerate() {
            match_result.add_trade(*trade);
            assert_eq!(
                match_result.executed_quantity(),
                expected_executed_quantities[i]
            );
            assert_eq!(match_result.executed_value(), expected_executed_values[i]);
        }
        assert_eq!(match_result.trades(), &trades);
    }
}
