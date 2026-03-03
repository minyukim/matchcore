use super::{CancelReason, Trade};
use crate::{Notional, OrderId, Price, Quantity, Side};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct OrderProcessingResults {
    /// Result for the primary order explicitly stated in the command
    primary_order: OrderProcessingResult,
    /// Other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    triggered_orders: Vec<OrderProcessingResult>,
}

impl OrderProcessingResults {
    /// Create a new order processing results
    pub(super) fn new(primary_order: OrderProcessingResult) -> Self {
        Self {
            primary_order,
            triggered_orders: Vec::new(),
        }
    }

    pub(super) fn set_triggered_orders(&mut self, triggered_orders: Vec<OrderProcessingResult>) {
        self.triggered_orders = triggered_orders;
    }

    /// Get the result for the primary order explicitly stated in the command
    pub(super) fn primary_order(&self) -> &OrderProcessingResult {
        &self.primary_order
    }

    /// Get the other orders whose state changed as a consequence (e.g., inactive pegged orders becoming active)
    pub(super) fn triggered_orders(&self) -> &[OrderProcessingResult] {
        &self.triggered_orders
    }
}

/// Result of processing a taker order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderProcessingResult {
    /// The ID of the order
    order_id: OrderId,
    /// The match result if the order was matched
    match_result: Option<MatchResult>,
    /// The reason the order was cancelled, if it was cancelled
    cancel_reason: Option<CancelReason>,
}

impl OrderProcessingResult {
    /// Create a new order processing result
    pub(crate) fn new(order_id: OrderId) -> Self {
        Self {
            order_id,
            match_result: None,
            cancel_reason: None,
        }
    }

    /// Return this order processing result with the match result set
    pub(crate) fn with_match_result(mut self, match_result: MatchResult) -> Self {
        self.match_result = Some(match_result);
        self
    }

    /// Return this order processing result with the reason the order was cancelled set
    pub(crate) fn with_cancel_reason(mut self, cancel_reason: CancelReason) -> Self {
        self.cancel_reason = Some(cancel_reason);
        self
    }

    /// Get the ID of the order
    pub fn order_id(&self) -> OrderId {
        self.order_id
    }

    /// Get the match result if the order was matched
    pub fn match_result(&self) -> Option<&MatchResult> {
        self.match_result.as_ref()
    }

    /// Get the reason the order was cancelled, if it was cancelled
    pub fn cancel_reason(&self) -> Option<&CancelReason> {
        self.cancel_reason.as_ref()
    }
}

#[cfg(test)]
mod tests_order_processing_result {
    use super::*;
    use crate::{
        Quantity, Side,
        report::{CancelReason, MatchResult},
    };

    fn create_order_processing_result() -> OrderProcessingResult {
        OrderProcessingResult::new(OrderId(1))
    }

    #[test]
    fn test_order_id() {
        assert_eq!(create_order_processing_result().order_id(), OrderId(1));
    }

    #[test]
    fn test_cancel_reason() {
        let mut order_processing_result = create_order_processing_result();
        assert_eq!(order_processing_result.cancel_reason(), None);

        let cancel_reason = CancelReason::InsufficientLiquidity {
            requested_quantity: Quantity(100),
            available_quantity: Quantity(50),
        };
        order_processing_result = order_processing_result.with_cancel_reason(cancel_reason.clone());
        assert_eq!(
            order_processing_result.cancel_reason(),
            Some(&cancel_reason)
        );
    }

    #[test]
    fn test_match_result() {
        let mut order_processing_result = create_order_processing_result();
        assert!(order_processing_result.match_result().is_none());

        let match_result = MatchResult::new(Side::Buy);
        order_processing_result = order_processing_result.with_match_result(match_result);
        assert!(order_processing_result.match_result().is_some());
    }
}

/// Result of a match operation
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests_match_result {
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
    }
}
