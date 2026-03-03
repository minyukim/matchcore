use crate::{
    LimitOrder, MatchResult, PegReference, PeggedOrder, Side, Trade,
    orderbook::{
        OrderBook, PegLevel, PriceLevel,
        peg_level::{MAKER_ARRAY_PRIMARY, MAKER_ARRAY_PRIMARY_MID_PRICE},
    },
};

use std::collections::{BTreeMap, HashMap};

impl OrderBook {
    /// Match an order against existing orders in the order book.
    ///
    /// It calls the `match_order` function to perform the matching without borrowing
    /// the entire order book mutably.
    ///
    /// Returns a `MatchResult` struct containing the result of the match.
    pub(super) fn match_order(
        &mut self,
        taker_side: Side,
        limit_price: Option<u64>,
        quantity: u64,
        timestamp: u64,
    ) -> MatchResult {
        let (taker_side_best_price, maker_side_price_levels, maker_side_peg_levels) =
            match taker_side {
                Side::Buy => (
                    self.best_bid(),
                    &mut self.limit_ask_levels,
                    &mut self.peg_ask_levels,
                ),
                Side::Sell => (
                    self.best_ask(),
                    &mut self.limit_bid_levels,
                    &mut self.peg_bid_levels,
                ),
            };

        let result = match_order(
            taker_side,
            taker_side_best_price,
            maker_side_price_levels,
            &mut self.limit_orders,
            maker_side_peg_levels,
            &mut self.pegged_orders,
            limit_price,
            quantity,
            timestamp,
        );
        self.last_trade_price = result.last_trade_price();

        result
    }

    /// Computes the immediately executable quantity against the current book,
    /// capped at `requested_quantity`, without mutating state.
    ///
    /// Preconditions:
    /// - `requested_quantity` > 0
    /// - has crossable order at `limit_price`
    ///
    /// Returns `requested_quantity` if fully executable; otherwise returns the
    /// available executable quantity.
    pub(super) fn max_executable_quantity_unchecked(
        &self,
        taker_side: Side,
        limit_price: u64,
        requested_quantity: u64,
    ) -> u64 {
        debug_assert!(requested_quantity > 0);
        debug_assert!(self.has_crossable_order(taker_side, limit_price));

        let mut remaining: u64 = requested_quantity;

        // MidPrice peg level is active if the spread is less than or equal to 1
        let mid_active = self.spread().is_some_and(|spread| spread <= 1);

        match taker_side {
            Side::Buy => {
                // Iterate over the limit ask price levels up to the limit price
                for (price, level) in self.limit_ask_levels.iter() {
                    if *price > limit_price {
                        break;
                    }
                    remaining = remaining.saturating_sub(level.total_quantity());
                    if remaining == 0 {
                        return requested_quantity;
                    }
                }
                // Primary peg level is always active
                remaining = remaining.saturating_sub(
                    self.peg_ask_levels[PegReference::Primary.as_index()].quantity(),
                );
                if remaining == 0 {
                    return requested_quantity;
                }
                if mid_active {
                    remaining = remaining.saturating_sub(
                        self.peg_ask_levels[PegReference::MidPrice.as_index()].quantity(),
                    );
                    if remaining == 0 {
                        return requested_quantity;
                    }
                }
            }
            Side::Sell => {
                // Iterate over the limit bid price levels up to the limit price
                for (price, level) in self.limit_bid_levels.iter().rev() {
                    if *price < limit_price {
                        break;
                    }
                    remaining = remaining.saturating_sub(level.total_quantity());
                    if remaining == 0 {
                        return requested_quantity;
                    }
                }
                // Primary peg level is always active
                remaining = remaining.saturating_sub(
                    self.peg_bid_levels[PegReference::Primary.as_index()].quantity(),
                );
                // MidPrice peg level is active if the spread is less than or equal to 1
                if mid_active {
                    remaining = remaining.saturating_sub(
                        self.peg_bid_levels[PegReference::MidPrice.as_index()].quantity(),
                    );
                    if remaining == 0 {
                        return requested_quantity;
                    }
                }
            }
        }

        requested_quantity - remaining
    }
}

/// Match an order against existing orders in the order book.
///
/// It first tries to find the best price level for the order, and consumes the orders in the price level.
/// If the price level is exhausted, it starts consuming the active peg levels.
/// If the active peg levels are exhausted, it moves to the next best price level,
/// and then active peg levels again, and so on.
/// It stops when the order is fully matched or all the levels are exhausted.
///
/// Returns a `MatchResult` struct containing the result of the match.
#[allow(clippy::too_many_arguments)]
pub(super) fn match_order(
    taker_side: Side,
    taker_side_best_price: Option<u64>,
    maker_side_price_levels: &mut BTreeMap<u64, PriceLevel>,
    limit_orders: &mut HashMap<u64, LimitOrder>,
    maker_side_peg_levels: &mut [PegLevel; PegReference::COUNT],
    pegged_orders: &mut HashMap<u64, PeggedOrder>,
    limit_price: Option<u64>,
    quantity: u64,
    timestamp: u64,
) -> MatchResult {
    let mut match_result = MatchResult::new(taker_side);
    let mut remaining_quantity = quantity;

    while remaining_quantity > 0 {
        let best_price = match taker_side {
            // The best price is the lowest price for a buy order (asks)
            Side::Buy => maker_side_price_levels.keys().next().copied(),
            // The best price is the highest price for a sell order (bids)
            Side::Sell => maker_side_price_levels.keys().next_back().copied(),
        };

        let Some(price) = best_price else {
            break;
        };

        if let Some(limit_price) = limit_price {
            match taker_side {
                Side::Buy if price > limit_price => break,
                Side::Sell if price < limit_price => break,
                _ => (),
            }
        }

        // The price level is guaranteed to exist because the best price is not None
        let price_level = maker_side_price_levels.get_mut(&price).unwrap();

        // Iterate over the orders at the price level
        while remaining_quantity > 0 {
            // The price level is guaranteed to have at least one order
            let order = price_level.peek(limit_orders).unwrap();
            let order_id = order.id();

            // The order is expired, remove it from the price level
            if order.is_expired(timestamp) {
                price_level.remove_head_order(limit_orders);
                if price_level.is_empty() {
                    maker_side_price_levels.remove(&price);
                    break;
                }
                continue;
            }

            let (consumed, replenished) = order.match_against(remaining_quantity);
            remaining_quantity -= consumed;

            price_level.consume(consumed);
            price_level.handle_replenishment(replenished);

            match_result.add_trade(Trade::new(order_id, price, consumed));

            // The order is filled, remove it from the price level
            if order.is_filled() {
                price_level.remove_head_order(limit_orders);
                if price_level.is_empty() {
                    maker_side_price_levels.remove(&price);
                    break;
                }
            }
        }

        // Determine the active peg references based on the taker side best price
        // Primary: always active
        // Market: not active
        // MidPrice: active if the price is within 1 of the taker side best price
        let active_peg_references: &[PegReference] = match taker_side_best_price {
            Some(taker_side_best_price) if price.abs_diff(taker_side_best_price) <= 1 => {
                &MAKER_ARRAY_PRIMARY_MID_PRICE
            }
            _ => &MAKER_ARRAY_PRIMARY,
        };

        // Iterate over the orders at the active peg levels
        while remaining_quantity > 0 {
            // (peg_level_index, order_id)
            let mut best: Option<(usize, u64)> = None;

            // Find the earliest order
            for peg_reference in active_peg_references {
                let idx = peg_reference.as_index();

                let candidate_id = {
                    let peg_level = &mut maker_side_peg_levels[idx];
                    peg_level.peek_order_id(pegged_orders)
                };

                let Some(candidate_id) = candidate_id else {
                    continue;
                };

                match best {
                    None => best = Some((idx, candidate_id)),
                    Some((_, best_id)) => {
                        if candidate_id < best_id {
                            best = Some((idx, candidate_id));
                        }
                    }
                }
            }

            // No more orders in the active peg levels
            let Some((best_level_idx, order_id)) = best else {
                break;
            };

            let peg_level = &mut maker_side_peg_levels[best_level_idx];

            // The order is guaranteed to exist because the order ID is found in the peg level
            let order = pegged_orders.get_mut(&order_id).unwrap();

            // The order is expired, remove it from the peg level
            if order.is_expired(timestamp) {
                peg_level.remove_head_order(pegged_orders);
                continue;
            }

            let consumed = order.match_against(remaining_quantity);
            remaining_quantity -= consumed;

            peg_level.consume(consumed);

            match_result.add_trade(Trade::new(order_id, price, consumed));

            // The order is filled, remove it from the peg level
            if order.is_filled() {
                peg_level.remove_head_order(pegged_orders);
            }
        }
    }

    match_result
}

#[cfg(test)]
mod tests_match_order {
    use super::*;
    use crate::{LimitOrderSpec, OrderFlags, QuantityPolicy, TimeInForce};

    /// Helper function to create a new test order book
    fn new_test_book() -> OrderBook {
        OrderBook::new("TEST")
    }

    /// Helper function to add a standard limit order to the book
    fn add_limit_order(book: &mut OrderBook, id: u64, price: u64, quantity: u64, side: Side) {
        book.add_limit_order(LimitOrder::new(
            id,
            LimitOrderSpec::new(
                price,
                QuantityPolicy::Standard { quantity },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        ));
    }

    #[test]
    fn test_single_maker_full_fill() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 50, Side::Sell);

        let result = orderbook.match_order(Side::Buy, Some(100), 50, 0);

        assert_eq!(result.taker_side(), Side::Buy);
        assert_eq!(result.executed_quantity(), 50);
        assert_eq!(result.executed_value(), 100 * 50);
        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].maker_order_id(), 0);
        assert_eq!(result.trades()[0].price(), 100);
        assert_eq!(result.trades()[0].quantity(), 50);

        assert_eq!(orderbook.last_trade_price(), Some(100));
        // Maker fully filled, level removed
        assert!(orderbook.best_ask().is_none());
    }

    #[test]
    fn test_single_maker_partial_fill() {
        let mut orderbook = new_test_book();
        assert!(orderbook.last_trade_price().is_none());
        assert!(orderbook.best_ask().is_none());

        // Add a sell order (maker) at 100 for 50
        add_limit_order(&mut orderbook, 0, 100, 50, Side::Sell);
        assert_eq!(orderbook.best_ask(), Some(100));

        // Match a buy order at 100 for 30 against the book
        let result = orderbook.match_order(Side::Buy, Some(100), 30, 0);

        assert_eq!(result.taker_side(), Side::Buy);
        assert_eq!(result.executed_quantity(), 30);
        assert_eq!(result.executed_value(), 100 * 30);
        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].maker_order_id(), 0);
        assert_eq!(result.trades()[0].price(), 100);
        assert_eq!(result.trades()[0].quantity(), 30);

        assert_eq!(orderbook.last_trade_price(), Some(100));
        // Best ask is still 100 with 20 remaining
        assert_eq!(orderbook.best_ask(), Some(100));

        // Match a buy order at 100 for 40 against the book
        let result = orderbook.match_order(Side::Buy, Some(100), 40, 0);

        assert_eq!(result.taker_side(), Side::Buy);
        assert_eq!(result.executed_quantity(), 20);
        assert_eq!(result.executed_value(), 100 * 20);
        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].maker_order_id(), 0);
        assert_eq!(result.trades()[0].price(), 100);
        assert_eq!(result.trades()[0].quantity(), 20);

        assert_eq!(orderbook.last_trade_price(), Some(100));
        // Maker fully filled, level removed
        assert!(orderbook.best_ask().is_none());
    }

    #[test]
    fn test_sell_taker() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 50, Side::Buy);

        let result = orderbook.match_order(Side::Sell, Some(100), 40, 0);

        assert_eq!(result.taker_side(), Side::Sell);
        assert_eq!(result.executed_quantity(), 40);
        assert_eq!(result.executed_value(), 100 * 40);
        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].maker_order_id(), 0);
        assert_eq!(result.trades()[0].price(), 100);
        assert_eq!(result.trades()[0].quantity(), 40);

        assert_eq!(orderbook.last_trade_price(), Some(100));
        // Best bid is still 100 with 10 remaining
        assert_eq!(orderbook.best_bid(), Some(100));

        // Match a sell order at 100 for 20 against the book
        let result = orderbook.match_order(Side::Sell, Some(100), 20, 0);

        assert_eq!(result.taker_side(), Side::Sell);
        assert_eq!(result.executed_quantity(), 10);
        assert_eq!(result.executed_value(), 100 * 10);
        assert_eq!(result.trades().len(), 1);
        assert_eq!(result.trades()[0].maker_order_id(), 0);
        assert_eq!(result.trades()[0].price(), 100);
        assert_eq!(result.trades()[0].quantity(), 10);

        assert_eq!(orderbook.last_trade_price(), Some(100));
        // Maker fully filled, level removed
        assert!(orderbook.best_bid().is_none());
    }

    #[test]
    fn test_empty_book_no_fill() {
        let mut orderbook = new_test_book();

        let result = orderbook.match_order(Side::Buy, None, 30, 0);

        assert_eq!(result.taker_side(), Side::Buy);
        assert_eq!(result.executed_quantity(), 0);
        assert_eq!(result.executed_value(), 0);
        assert!(result.trades().is_empty());
        assert!(orderbook.last_trade_price().is_none());
    }

    #[test]
    fn test_limit_not_crossed_no_fill() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 50, Side::Sell);

        // Buy limit 99 does not cross best ask 100
        let result = orderbook.match_order(Side::Buy, Some(99), 30, 0);

        assert_eq!(result.executed_quantity(), 0);
        assert!(result.trades().is_empty());
        assert_eq!(orderbook.best_ask(), Some(100));
    }

    #[test]
    fn test_multiple_makers_same_price() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 20, Side::Sell);
        add_limit_order(&mut orderbook, 1, 100, 30, Side::Sell);

        // Buy 40: fills first maker fully (20), second maker partially (20)
        let result = orderbook.match_order(Side::Buy, Some(100), 40, 0);

        assert_eq!(result.executed_quantity(), 40);
        assert_eq!(result.executed_value(), 100 * 40);
        assert_eq!(result.trades().len(), 2);
        assert_eq!(result.trades()[0].maker_order_id(), 0);
        assert_eq!(result.trades()[0].price(), 100);
        assert_eq!(result.trades()[0].quantity(), 20);
        assert_eq!(result.trades()[1].maker_order_id(), 1);
        assert_eq!(result.trades()[1].price(), 100);
        assert_eq!(result.trades()[1].quantity(), 20);
        assert_eq!(orderbook.last_trade_price(), Some(100));
        // Second maker has 10 left at 100
        assert_eq!(orderbook.best_ask(), Some(100));
    }

    #[test]
    fn test_multiple_price_levels() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 30, Side::Sell);
        add_limit_order(&mut orderbook, 1, 101, 40, Side::Sell);

        // Buy 50 at limit 101: 30 @ 100, then 20 @ 101
        let result = orderbook.match_order(Side::Buy, Some(101), 50, 0);

        assert_eq!(result.executed_quantity(), 50);
        assert_eq!(result.executed_value(), 30 * 100 + 20 * 101);
        assert_eq!(result.trades().len(), 2);
        assert_eq!(result.trades()[0].maker_order_id(), 0);
        assert_eq!(result.trades()[0].price(), 100);
        assert_eq!(result.trades()[0].quantity(), 30);
        assert_eq!(result.trades()[1].maker_order_id(), 1);
        assert_eq!(result.trades()[1].price(), 101);
        assert_eq!(result.trades()[1].quantity(), 20);
        assert_eq!(orderbook.last_trade_price(), Some(101));
        // Best ask now 101 with 20 remaining
        assert_eq!(orderbook.best_ask(), Some(101));
    }

    #[test]
    fn test_market_buy_sweeps_levels() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 25, Side::Sell);
        add_limit_order(&mut orderbook, 1, 101, 25, Side::Sell);
        add_limit_order(&mut orderbook, 2, 102, 25, Side::Sell);

        // Market buy (None limit) for 60: 25 @ 100, 25 @ 101, 10 @ 102
        let result = orderbook.match_order(Side::Buy, None, 60, 0);

        assert_eq!(result.executed_quantity(), 60);
        assert_eq!(result.executed_value(), 25 * 100 + 25 * 101 + 10 * 102);
        assert_eq!(result.trades().len(), 3);
        assert_eq!(result.trades()[0], Trade::new(0, 100, 25));
        assert_eq!(result.trades()[1], Trade::new(1, 101, 25));
        assert_eq!(result.trades()[2], Trade::new(2, 102, 10));
        assert_eq!(orderbook.last_trade_price(), Some(102));
        assert_eq!(orderbook.best_ask(), Some(102));
    }
}

#[cfg(test)]
mod tests_max_executable_quantity_unchecked {
    use super::*;
    use crate::{LimitOrderSpec, OrderFlags, QuantityPolicy, TimeInForce};

    /// Helper function to create a new test order book
    fn new_test_book() -> OrderBook {
        OrderBook::new("TEST")
    }

    /// Helper function to add a standard limit order to the book
    fn add_limit_order(book: &mut OrderBook, id: u64, price: u64, quantity: u64, side: Side) {
        book.add_limit_order(LimitOrder::new(
            id,
            LimitOrderSpec::new(
                price,
                QuantityPolicy::Standard { quantity },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        ));
    }

    #[test]
    fn test_fully_executable_returns_requested() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 50, Side::Sell);

        // Buy 30 at 100: 30 available, request 30 → fully executable
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, 100, 30);
        assert_eq!(qty, 30);
    }

    #[test]
    fn test_capped_by_available_liquidity() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 50, Side::Sell);

        // Buy 100 at 100: only 50 available
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, 100, 100);
        assert_eq!(qty, 50);
    }

    #[test]
    fn test_multiple_levels_summed_up_to_limit_price() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 30, Side::Sell);
        add_limit_order(&mut orderbook, 1, 101, 40, Side::Sell);

        // Buy at limit 101: 30 + 40 = 70 available, request 100 → 70 executable
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, 101, 100);
        assert_eq!(qty, 70);
    }

    #[test]
    fn test_buy_respects_limit_price_ceiling() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 10, Side::Sell);
        add_limit_order(&mut orderbook, 1, 102, 20, Side::Sell);

        // Buy at limit 101: only 10 @ 100 counts, 102 is above limit
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, 101, 100);
        assert_eq!(qty, 10);
    }

    #[test]
    fn test_sell_taker_fully_executable() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 50, Side::Buy);

        // Sell 30 at 100: 30 executable
        let qty = orderbook.max_executable_quantity_unchecked(Side::Sell, 100, 30);
        assert_eq!(qty, 30);
    }

    #[test]
    fn test_sell_taker_capped_by_bids() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 100, 50, Side::Buy);

        // Sell 100 at 100: only 50 available
        let qty = orderbook.max_executable_quantity_unchecked(Side::Sell, 100, 100);
        assert_eq!(qty, 50);
    }

    #[test]
    fn test_sell_respects_limit_price_floor() {
        let mut orderbook = new_test_book();
        add_limit_order(&mut orderbook, 0, 98, 30, Side::Buy);
        add_limit_order(&mut orderbook, 1, 100, 50, Side::Buy);

        // Sell at limit 99: only 50 @ 100 counts (bid >= 99), 98 is below limit
        let qty = orderbook.max_executable_quantity_unchecked(Side::Sell, 99, 100);
        assert_eq!(qty, 50);
    }
}
