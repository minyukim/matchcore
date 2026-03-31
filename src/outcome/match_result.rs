use super::Trade;
use crate::{Notional, Price, Quantity, Side};

use std::fmt;

/// Result of a match operation
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    /// The side of the taker order
    taker_side: Side,
    /// The total executed quantity during the match
    executed_quantity: Quantity,
    /// The total value of the trades made during the match
    executed_value: Notional,
    /// The trades that were made during the match
    trades: Vec<Trade>,
}

impl MatchResult {
    /// Create a new match result
    pub(crate) fn new(taker_side: Side) -> Self {
        Self {
            taker_side,
            executed_quantity: Quantity(0),
            executed_value: Notional(0),
            trades: Vec::new(),
        }
    }

    /// Get the side of the taker order
    pub fn taker_side(&self) -> Side {
        self.taker_side
    }

    /// Get the total executed quantity during the match
    pub fn executed_quantity(&self) -> Quantity {
        self.executed_quantity
    }

    /// Get the total value of the trades made during the match
    pub fn executed_value(&self) -> Notional {
        self.executed_value
    }

    /// Get the trades that were made during the match
    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    /// Get the price of the first trade made during the match
    pub fn first_trade_price(&self) -> Option<Price> {
        self.trades.first().map(|trade| trade.price())
    }

    /// Get the price of the last trade made during the match
    pub fn last_trade_price(&self) -> Option<Price> {
        self.trades.last().map(|trade| trade.price())
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

impl fmt::Display for MatchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "taker_side={} executed_quantity={} executed_value={} trades={}",
            self.taker_side(),
            self.executed_quantity(),
            self.executed_value(),
            self.trades().len(),
        )?;

        for trade in self.trades() {
            writeln!(f, "  {}", trade)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_match_result {
    use super::*;
    use crate::{OrderId, Side};

    fn create_match_result() -> MatchResult {
        MatchResult::new(Side::Buy)
    }

    #[test]
    fn test_taker_side() {
        assert_eq!(create_match_result().taker_side(), Side::Buy);
    }

    #[test]
    fn test_executed_quantity() {
        assert_eq!(create_match_result().executed_quantity(), Quantity(0));
    }

    #[test]
    fn test_executed_value() {
        assert_eq!(create_match_result().executed_value(), Notional(0));
    }

    #[test]
    fn test_trades() {
        let mut match_result = create_match_result();
        assert_eq!(match_result.trades(), &[]);

        let trades = [
            Trade::new(OrderId(2), Price(99), Quantity(20)),
            Trade::new(OrderId(3), Price(100), Quantity(30)),
            Trade::new(OrderId(4), Price(101), Quantity(20)),
        ];

        let expected_executed_quantities = [Quantity(20), Quantity(50), Quantity(70)];
        let expected_executed_values = [Notional(1980), Notional(4980), Notional(7000)];

        for (i, trade) in trades.iter().enumerate() {
            match_result.add_trade(*trade);
            assert_eq!(
                match_result.executed_quantity(),
                expected_executed_quantities[i]
            );
            assert_eq!(match_result.executed_value(), expected_executed_values[i]);
        }
        assert_eq!(match_result.trades(), &trades);
        assert_eq!(match_result.first_trade_price(), Some(Price(99)));
        assert_eq!(match_result.last_trade_price(), Some(Price(101)));
    }

    #[test]
    fn test_display() {
        let mut match_result = create_match_result();
        println!("{}", match_result);
        assert_eq!(
            match_result.to_string(),
            "taker_side=BUY executed_quantity=0 executed_value=0 trades=0\n"
        );

        match_result.add_trade(Trade::new(OrderId(2), Price(99), Quantity(20)));
        match_result.add_trade(Trade::new(OrderId(3), Price(100), Quantity(30)));
        println!("{}", match_result);
        assert_eq!(
            match_result.to_string(),
            "taker_side=BUY executed_quantity=50 executed_value=4980 trades=2\n  maker(2): 20@99\n  maker(3): 30@100\n"
        );
    }
}
