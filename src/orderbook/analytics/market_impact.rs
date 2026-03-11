use crate::{Notional, OrderBook, PegReference, Price, Quantity, Side};

use serde::{Deserialize, Serialize};

/// Represents the market impact analysis of an order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketImpact {
    /// Requested quantity of the order
    requested_quantity: Quantity,

    /// Total quantity available to fill the order
    available_quantity: Quantity,

    /// Total cost to fill the order
    total_cost: Notional,

    /// Best execution price
    best_price: Price,

    /// Worst (furthest from the best price) execution price
    worst_price: Price,

    /// Number of price levels that would be consumed
    consumed_price_levels: usize,
}

impl MarketImpact {
    /// Compute the market impact of a market order
    ///
    /// Included quantities: visible, hidden, and active peg levels
    pub(super) fn compute(book: &OrderBook, taker_side: Side, quantity: Quantity) -> Self {
        let mut impact = Self {
            requested_quantity: quantity,
            available_quantity: Quantity(0),
            total_cost: Notional(0),
            best_price: Price(0),
            worst_price: Price(0),
            consumed_price_levels: 0,
        };

        if quantity.is_zero() {
            return impact;
        }

        let mut remaining = quantity;

        // MidPrice peg level is active if the spread is less than or equal to 1
        let mid_active = book.spread().is_some_and(|spread| spread <= 1);

        match taker_side {
            Side::Buy => {
                // No liquidity
                let Some(best_ask) = book.best_ask_price() else {
                    return impact;
                };
                impact.best_price = best_ask;

                if mid_active {
                    let available =
                        book.pegged.ask_levels[PegReference::MidPrice.as_index()].quantity();
                    let fill_qty = remaining.min(available);
                    impact.total_cost = impact.total_cost.saturating_add(best_ask * fill_qty);
                    impact.available_quantity = impact.available_quantity.saturating_add(fill_qty);

                    remaining = remaining.saturating_sub(fill_qty);
                    if remaining.is_zero() {
                        impact.worst_price = best_ask;
                        impact.consumed_price_levels = 1;
                        return impact;
                    }
                }
                // Primary peg level is always active
                let available = book.pegged.ask_levels[PegReference::Primary.as_index()].quantity();
                let fill_qty = remaining.min(available);
                impact.total_cost = impact.total_cost.saturating_add(best_ask * fill_qty);
                impact.available_quantity = impact.available_quantity.saturating_add(fill_qty);

                remaining = remaining.saturating_sub(fill_qty);
                if remaining.is_zero() {
                    impact.worst_price = best_ask;
                    impact.consumed_price_levels = 1;
                    return impact;
                }

                // Iterate over the limit ask price levels
                for (price, level) in book.limit.ask_levels.iter() {
                    let available = level.total_quantity();
                    let fill_qty = remaining.min(available);
                    impact.total_cost = impact.total_cost.saturating_add(*price * fill_qty);
                    impact.available_quantity = impact.available_quantity.saturating_add(fill_qty);
                    impact.worst_price = *price;
                    impact.consumed_price_levels += 1;

                    remaining = remaining.saturating_sub(fill_qty);
                    if remaining.is_zero() {
                        return impact;
                    }
                }

                impact
            }
            Side::Sell => {
                // No liquidity
                let Some(best_bid) = book.best_bid_price() else {
                    return impact;
                };
                impact.best_price = best_bid;

                if mid_active {
                    let available =
                        book.pegged.bid_levels[PegReference::MidPrice.as_index()].quantity();
                    let fill_qty = remaining.min(available);
                    impact.total_cost = impact.total_cost.saturating_add(best_bid * fill_qty);
                    impact.available_quantity = impact.available_quantity.saturating_add(fill_qty);

                    remaining = remaining.saturating_sub(fill_qty);
                    if remaining.is_zero() {
                        impact.worst_price = best_bid;
                        impact.consumed_price_levels = 1;
                        return impact;
                    }
                }
                // Primary peg level is always active
                let available = book.pegged.bid_levels[PegReference::Primary.as_index()].quantity();
                let fill_qty = remaining.min(available);
                impact.total_cost = impact.total_cost.saturating_add(best_bid * fill_qty);
                impact.available_quantity = impact.available_quantity.saturating_add(fill_qty);

                remaining = remaining.saturating_sub(fill_qty);
                if remaining.is_zero() {
                    impact.worst_price = best_bid;
                    impact.consumed_price_levels = 1;
                    return impact;
                }

                // Iterate over the limit ask price levels
                for (price, level) in book.limit.bid_levels.iter().rev() {
                    let available = level.total_quantity();
                    let fill_qty = remaining.min(available);
                    impact.total_cost = impact.total_cost.saturating_add(*price * fill_qty);
                    impact.available_quantity = impact.available_quantity.saturating_add(fill_qty);
                    impact.worst_price = *price;
                    impact.consumed_price_levels += 1;

                    remaining = remaining.saturating_sub(fill_qty);
                    if remaining.is_zero() {
                        return impact;
                    }
                }

                impact
            }
        }
    }

    /// Get the requested quantity of the order
    pub fn requested_quantity(&self) -> Quantity {
        self.requested_quantity
    }

    /// Get the available quantity to fill the order
    pub fn available_quantity(&self) -> Quantity {
        self.available_quantity
    }

    /// Get the total cost to fill the order
    pub fn total_cost(&self) -> Notional {
        self.total_cost
    }

    /// Get the best execution price
    pub fn best_price(&self) -> Price {
        self.best_price
    }

    /// Get the worst (furthest from the best price) execution price
    pub fn worst_price(&self) -> Price {
        self.worst_price
    }

    /// Get the number of price levels that would be consumed
    pub fn consumed_price_levels(&self) -> usize {
        self.consumed_price_levels
    }

    /// Get the average price to fill the order
    pub fn average_price(&self) -> f64 {
        if self.available_quantity.is_zero() {
            return 0.0;
        }
        self.total_cost / self.available_quantity
    }

    /// Get the slippage from the best to the worst price
    pub fn slippage(&self) -> u64 {
        self.best_price.abs_diff(self.worst_price)
    }
}
