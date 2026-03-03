use super::{
    OrderBook, PegLevel, PriceLevel,
    peg_level::{MAKER_ARRAY_PRIMARY, MAKER_ARRAY_PRIMARY_MID_PRICE},
};
use crate::{
    LimitOrder, MatchResult, OrderId, PegReference, PeggedOrder, Price, Quantity, Side, Timestamp,
    Trade,
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
        limit_price: Option<Price>,
        quantity: Quantity,
        timestamp: Timestamp,
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
        limit_price: Price,
        requested_quantity: Quantity,
    ) -> Quantity {
        debug_assert!(!requested_quantity.is_zero());
        debug_assert!(self.has_crossable_order(taker_side, limit_price));

        let mut remaining = requested_quantity;

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
                    if remaining.is_zero() {
                        return requested_quantity;
                    }
                }
                // Primary peg level is always active
                remaining = remaining.saturating_sub(
                    self.peg_ask_levels[PegReference::Primary.as_index()].quantity(),
                );
                if remaining.is_zero() {
                    return requested_quantity;
                }
                if mid_active {
                    remaining = remaining.saturating_sub(
                        self.peg_ask_levels[PegReference::MidPrice.as_index()].quantity(),
                    );
                    if remaining.is_zero() {
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
                    if remaining.is_zero() {
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
                    if remaining.is_zero() {
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
    taker_side_best_price: Option<Price>,
    maker_side_price_levels: &mut BTreeMap<Price, PriceLevel>,
    limit_orders: &mut HashMap<OrderId, LimitOrder>,
    maker_side_peg_levels: &mut [PegLevel; PegReference::COUNT],
    pegged_orders: &mut HashMap<OrderId, PeggedOrder>,
    limit_price: Option<Price>,
    quantity: Quantity,
    timestamp: Timestamp,
) -> MatchResult {
    let mut match_result = MatchResult::new(taker_side);
    let mut remaining_quantity = quantity;

    while !remaining_quantity.is_zero() {
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
        while !remaining_quantity.is_zero() {
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
        while !remaining_quantity.is_zero() {
            // (peg_level_index, order_id)
            let mut best: Option<(usize, OrderId)> = None;

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
    use crate::{LimitOrderSpec, Notional, OrderFlags, Quantity, QuantityPolicy, TimeInForce};

    /// Helper function to create a new test order book
    fn new_test_book() -> OrderBook {
        OrderBook::new("TEST")
    }

    /// Helper function to add a standard limit order to the book
    fn add_standard_order(
        book: &mut OrderBook,
        id: OrderId,
        price: Price,
        quantity: Quantity,
        side: Side,
    ) {
        book.add_limit_order(LimitOrder::new(
            id,
            LimitOrderSpec::new(
                price,
                QuantityPolicy::Standard { quantity },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        ));
    }

    /// Helper function to add an iceberg limit order to the book
    fn add_iceberg_order(
        book: &mut OrderBook,
        id: OrderId,
        price: Price,
        visible_quantity: Quantity,
        hidden_quantity: Quantity,
        replenish_quantity: Quantity,
        side: Side,
    ) {
        book.add_limit_order(LimitOrder::new(
            id,
            LimitOrderSpec::new(
                price,
                QuantityPolicy::Iceberg {
                    visible_quantity,
                    hidden_quantity,
                    replenish_quantity,
                },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        ));
    }

    #[test]
    fn test_single_maker_full_fill() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        let result = orderbook.match_order(Side::Buy, Some(Price(100)), Quantity(50), Timestamp(0));

        assert_eq!(result.taker_side(), Side::Buy);
        assert_eq!(result.executed_quantity(), Quantity(50));
        assert_eq!(result.executed_value(), Notional(100 * 50));
        assert_eq!(result.trades().len(), 1);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(50))
        );

        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Maker fully filled, level removed
        assert!(orderbook.best_ask().is_none());
    }

    #[test]
    fn test_single_maker_partial_fill() {
        let mut orderbook = new_test_book();
        assert!(orderbook.last_trade_price().is_none());
        assert!(orderbook.best_ask().is_none());

        // Add a sell order (maker) at 100 for 50
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );
        assert_eq!(orderbook.best_ask(), Some(Price(100)));

        // Match a buy order at 100 for 30 against the book
        let result = orderbook.match_order(Side::Buy, Some(Price(100)), Quantity(30), Timestamp(0));

        assert_eq!(result.taker_side(), Side::Buy);
        assert_eq!(result.executed_quantity(), Quantity(30));
        assert_eq!(result.executed_value(), Notional(100 * 30));
        assert_eq!(result.trades().len(), 1);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(30))
        );

        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Best ask is still 100 with 20 remaining
        assert_eq!(orderbook.best_ask(), Some(Price(100)));

        // Match a buy order at 100 for 40 against the book
        let result = orderbook.match_order(Side::Buy, Some(Price(100)), Quantity(40), Timestamp(0));

        assert_eq!(result.taker_side(), Side::Buy);
        assert_eq!(result.executed_quantity(), Quantity(20));
        assert_eq!(result.executed_value(), Notional(100 * 20));
        assert_eq!(result.trades().len(), 1);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(20))
        );

        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Maker fully filled, level removed
        assert!(orderbook.best_ask().is_none());
    }

    #[test]
    fn test_sell_taker() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        let result =
            orderbook.match_order(Side::Sell, Some(Price(100)), Quantity(40), Timestamp(0));

        assert_eq!(result.taker_side(), Side::Sell);
        assert_eq!(result.executed_quantity(), Quantity(40));
        assert_eq!(result.executed_value(), Notional(100 * 40));
        assert_eq!(result.trades().len(), 1);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(40))
        );

        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Best bid is still 100 with 10 remaining
        assert_eq!(orderbook.best_bid(), Some(Price(100)));

        // Match a sell order at 100 for 20 against the book
        let result =
            orderbook.match_order(Side::Sell, Some(Price(100)), Quantity(20), Timestamp(0));

        assert_eq!(result.taker_side(), Side::Sell);
        assert_eq!(result.executed_quantity(), Quantity(10));
        assert_eq!(result.executed_value(), Notional(100 * 10));
        assert_eq!(result.trades().len(), 1);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );

        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Maker fully filled, level removed
        assert!(orderbook.best_bid().is_none());
    }

    #[test]
    fn test_empty_book_no_fill() {
        let mut orderbook = new_test_book();

        let result = orderbook.match_order(Side::Buy, None, Quantity(30), Timestamp(0));

        assert_eq!(result.taker_side(), Side::Buy);
        assert_eq!(result.executed_quantity(), Quantity(0));
        assert_eq!(result.executed_value(), Notional(0));
        assert!(result.trades().is_empty());
        assert!(orderbook.last_trade_price().is_none());
    }

    #[test]
    fn test_limit_not_crossed_no_fill() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        // Buy limit 99 does not cross best ask 100
        let result = orderbook.match_order(Side::Buy, Some(Price(99)), Quantity(30), Timestamp(0));

        assert_eq!(result.executed_quantity(), Quantity(0));
        assert!(result.trades().is_empty());
        assert_eq!(orderbook.best_ask(), Some(Price(100)));
    }

    #[test]
    fn test_multiple_makers_same_price() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(20),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(1),
            Price(100),
            Quantity(30),
            Side::Sell,
        );

        // Buy 40: fills first maker fully (20), second maker partially (20)
        let result = orderbook.match_order(Side::Buy, Some(Price(100)), Quantity(40), Timestamp(0));

        assert_eq!(result.executed_quantity(), Quantity(40));
        assert_eq!(result.executed_value(), Notional(100 * 40));
        assert_eq!(result.trades().len(), 2);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(20))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(1), Price(100), Quantity(20))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Second maker has 10 left at 100
        assert_eq!(orderbook.best_ask(), Some(Price(100)));
    }

    #[test]
    fn test_multiple_price_levels() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(30),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(1),
            Price(101),
            Quantity(40),
            Side::Sell,
        );

        // Buy 50 at limit 101: 30 @ 100, then 20 @ 101
        let result = orderbook.match_order(Side::Buy, Some(Price(101)), Quantity(50), Timestamp(0));

        assert_eq!(result.executed_quantity(), Quantity(50));
        assert_eq!(result.executed_value(), Notional(30 * 100 + 20 * 101));
        assert_eq!(result.trades().len(), 2);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(30))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(1), Price(101), Quantity(20))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(101)));
        // Best ask now 101 with 20 remaining
        assert_eq!(orderbook.best_ask(), Some(Price(101)));
    }

    #[test]
    fn test_market_buy_sweeps_levels() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(25),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(1),
            Price(101),
            Quantity(25),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(2),
            Price(102),
            Quantity(25),
            Side::Sell,
        );

        // Market buy (None limit) for 60: 25 @ 100, 25 @ 101, 10 @ 102
        let result = orderbook.match_order(Side::Buy, None, Quantity(60), Timestamp(0));

        assert_eq!(result.executed_quantity(), Quantity(60));
        assert_eq!(
            result.executed_value(),
            Notional(25 * 100 + 25 * 101 + 10 * 102)
        );
        assert_eq!(result.trades().len(), 3);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(25))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(1), Price(101), Quantity(25))
        );
        assert_eq!(
            result.trades()[2],
            Trade::new(OrderId(2), Price(102), Quantity(10))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(102)));
        assert_eq!(orderbook.best_ask(), Some(Price(102)));
    }

    // --- Iceberg test cases ---

    #[test]
    fn test_iceberg_maker_partial_fill_visible_only() {
        let mut orderbook = new_test_book();
        // Iceberg: visible 20, hidden 30, replenish 10 (total 50)
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(20),
            Quantity(30),
            Quantity(10),
            Side::Sell,
        );

        // Buy 15: only consumes visible, no replenish yet
        let result = orderbook.match_order(Side::Buy, Some(Price(100)), Quantity(15), Timestamp(0));

        assert_eq!(result.executed_quantity(), Quantity(15));
        assert_eq!(result.executed_value(), Notional(100 * 15));
        assert_eq!(result.trades().len(), 1);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(15))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Iceberg still has 5 visible + 30 hidden at 100
        assert_eq!(orderbook.best_ask(), Some(Price(100)));
    }

    #[test]
    fn test_iceberg_maker_replenish_during_match() {
        let mut orderbook = new_test_book();
        // Iceberg: visible 10, hidden 20, replenish 10
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );

        // Buy 15: consumes 10 (trade 10), replenish 10, then consumes 5 (trade 5)
        let result = orderbook.match_order(Side::Buy, Some(Price(100)), Quantity(15), Timestamp(0));

        assert_eq!(result.executed_quantity(), Quantity(15));
        assert_eq!(result.executed_value(), Notional(100 * 15));
        assert_eq!(result.trades().len(), 2);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(0), Price(100), Quantity(5))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Iceberg has 5 visible + 10 hidden left
        assert_eq!(orderbook.best_ask(), Some(Price(100)));
    }

    #[test]
    fn test_iceberg_maker_multiple_replenishes_in_one_match() {
        let mut orderbook = new_test_book();
        // Iceberg: visible 10, hidden 30, replenish 10 (total 40)
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(30),
            Quantity(10),
            Side::Sell,
        );

        // Buy 35: 10 + 10 (replenish) + 10 + 5
        let result = orderbook.match_order(Side::Buy, Some(Price(100)), Quantity(35), Timestamp(0));

        assert_eq!(result.executed_quantity(), Quantity(35));
        assert_eq!(result.executed_value(), Notional(100 * 35));
        assert_eq!(result.trades().len(), 4);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[2],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[3],
            Trade::new(OrderId(0), Price(100), Quantity(5))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Iceberg has 5 visible left
        assert_eq!(orderbook.best_ask(), Some(Price(100)));
    }

    #[test]
    fn test_iceberg_maker_fully_filled() {
        let mut orderbook = new_test_book();
        // Iceberg: visible 10, hidden 20, replenish 10 (total 30)
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );

        let result = orderbook.match_order(Side::Buy, Some(Price(100)), Quantity(30), Timestamp(0));

        assert_eq!(result.executed_quantity(), Quantity(30));
        assert_eq!(result.executed_value(), Notional(100 * 30));
        assert_eq!(result.trades().len(), 3);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[2],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Iceberg fully filled, level removed
        assert!(orderbook.best_ask().is_none());
    }

    #[test]
    fn test_iceberg_then_standard_same_price() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(1),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        // Buy 70: replenish moves iceberg to back, so we get 10 (iceberg), then 50 (standard), then 10 (iceberg) = 3 trades
        let result = orderbook.match_order(Side::Buy, Some(Price(100)), Quantity(70), Timestamp(0));

        assert_eq!(result.executed_quantity(), Quantity(70));
        assert_eq!(result.executed_value(), Notional(100 * 70));
        assert_eq!(result.trades().len(), 3);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(1), Price(100), Quantity(50))
        );
        assert_eq!(
            result.trades()[2],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Iceberg has 10 visible left (replenished); standard fully filled
        assert_eq!(orderbook.best_ask(), Some(Price(100)));
    }

    #[test]
    fn test_iceberg_sell_taker_against_bids() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Buy,
        );

        // Sell 25: 10 + 10 (replenish) + 5
        let result =
            orderbook.match_order(Side::Sell, Some(Price(100)), Quantity(25), Timestamp(0));

        assert_eq!(result.taker_side(), Side::Sell);
        assert_eq!(result.executed_quantity(), Quantity(25));
        assert_eq!(result.executed_value(), Notional(100 * 25));
        assert_eq!(result.trades().len(), 3);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[2],
            Trade::new(OrderId(0), Price(100), Quantity(5))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Iceberg bid has 5 visible left
        assert_eq!(orderbook.best_bid(), Some(Price(100)));
    }
}

#[cfg(test)]
mod tests_max_executable_quantity_unchecked {
    use super::*;
    use crate::{LimitOrderSpec, OrderFlags, Quantity, QuantityPolicy, TimeInForce};

    /// Helper function to create a new test order book
    fn new_test_book() -> OrderBook {
        OrderBook::new("TEST")
    }

    /// Helper function to add a standard limit order to the book
    fn add_standard_order(
        book: &mut OrderBook,
        id: OrderId,
        price: Price,
        quantity: Quantity,
        side: Side,
    ) {
        book.add_limit_order(LimitOrder::new(
            id,
            LimitOrderSpec::new(
                price,
                QuantityPolicy::Standard { quantity },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        ));
    }

    /// Helper function to add an iceberg limit order to the book
    fn add_iceberg_order(
        book: &mut OrderBook,
        id: OrderId,
        price: Price,
        visible_quantity: Quantity,
        hidden_quantity: Quantity,
        replenish_quantity: Quantity,
        side: Side,
    ) {
        book.add_limit_order(LimitOrder::new(
            id,
            LimitOrderSpec::new(
                price,
                QuantityPolicy::Iceberg {
                    visible_quantity,
                    hidden_quantity,
                    replenish_quantity,
                },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        ));
    }

    #[test]
    fn test_fully_executable_returns_requested() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        // Buy 30 at 100: 30 available, request 30 → fully executable
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, Price(100), Quantity(30));
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_capped_by_available_liquidity() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        // Buy 100 at 100: only 50 available
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, Price(100), Quantity(100));
        assert_eq!(qty, Quantity(50));
    }

    #[test]
    fn test_multiple_levels_summed_up_to_limit_price() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(30),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(1),
            Price(101),
            Quantity(40),
            Side::Sell,
        );

        // Buy at limit 101: 30 + 40 = 70 available, request 100 → 70 executable
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, Price(101), Quantity(100));
        assert_eq!(qty, Quantity(70));
    }

    #[test]
    fn test_buy_respects_limit_price_ceiling() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(1),
            Price(102),
            Quantity(20),
            Side::Sell,
        );

        // Buy at limit 101: only 10 @ 100 counts, 102 is above limit
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, Price(101), Quantity(100));
        assert_eq!(qty, Quantity(10));
    }

    #[test]
    fn test_sell_taker_fully_executable() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        // Sell 30 at 100: 30 executable
        let qty = orderbook.max_executable_quantity_unchecked(Side::Sell, Price(100), Quantity(30));
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_sell_taker_capped_by_bids() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        // Sell 100 at 100: only 50 available
        let qty =
            orderbook.max_executable_quantity_unchecked(Side::Sell, Price(100), Quantity(100));
        assert_eq!(qty, Quantity(50));
    }

    #[test]
    fn test_sell_respects_limit_price_floor() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(98),
            Quantity(30),
            Side::Buy,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(1),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        // Sell at limit 99: only 50 @ 100 counts (bid >= 99), 98 is below limit
        let qty = orderbook.max_executable_quantity_unchecked(Side::Sell, Price(99), Quantity(100));
        assert_eq!(qty, Quantity(50));
    }

    // --- Iceberg test cases (total = visible + hidden at each level) ---

    #[test]
    fn test_iceberg_buy_capped_by_total_quantity() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );

        // Buy at 100: executable = visible + hidden = 30, request 50 → 30
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, Price(100), Quantity(50));
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_iceberg_buy_fully_executable() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(25),
            Quantity(10),
            Side::Sell,
        );

        // Buy at 100: 35 total available, request 20 → 20 (fully executable)
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, Price(100), Quantity(20));
        assert_eq!(qty, Quantity(20));
    }

    #[test]
    fn test_iceberg_sell_capped_by_total_quantity() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(15),
            Quantity(25),
            Quantity(10),
            Side::Buy,
        );

        // Sell at 100: executable = 40 total, request 50 → 40
        let qty = orderbook.max_executable_quantity_unchecked(Side::Sell, Price(100), Quantity(50));
        assert_eq!(qty, Quantity(40));
    }

    #[test]
    fn test_iceberg_and_standard_levels_summed() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(1),
            Price(101),
            Quantity(40),
            Side::Sell,
        );

        // Buy at 101: 30 (iceberg) + 40 (standard) = 70, request 100 → 70
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, Price(101), Quantity(100));
        assert_eq!(qty, Quantity(70));
    }

    #[test]
    fn test_iceberg_respects_buy_limit_price_ceiling() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            OrderId(1),
            Price(102),
            Quantity(50),
            Side::Sell,
        );

        // Buy at limit 101: only 30 @ 100 counts, 102 is above limit
        let qty = orderbook.max_executable_quantity_unchecked(Side::Buy, Price(101), Quantity(100));
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_iceberg_respects_sell_limit_price_floor() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            OrderId(0),
            Price(98),
            Quantity(30),
            Side::Buy,
        );
        add_iceberg_order(
            &mut orderbook,
            OrderId(1),
            Price(100),
            Quantity(10),
            Quantity(40),
            Quantity(10),
            Side::Buy,
        );

        // Sell at limit 99: only 50 @ 100 (iceberg total) counts, 98 is below limit
        let qty = orderbook.max_executable_quantity_unchecked(Side::Sell, Price(99), Quantity(100));
        assert_eq!(qty, Quantity(50));
    }
}
