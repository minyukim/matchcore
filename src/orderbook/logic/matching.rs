use crate::{orderbook::*, orders::*, outcome::*, types::*};

use std::{cmp::max, collections::BTreeMap};

use rustc_hash::FxHashMap;
use slab::Slab;

impl OrderBook {
    /// Match an order against existing orders in the order book.
    ///
    /// It calls the `match_order` function to perform the matching without borrowing
    /// the entire order book mutably.
    ///
    /// Returns a `MatchResult` struct containing the result of the match.
    pub(crate) fn match_order(
        &mut self,
        sequence_number: SequenceNumber,
        taker_side: Side,
        limit_price: Option<Price>,
        quantity: Quantity,
    ) -> MatchResult {
        let (
            taker_side_best_price,
            maker_side_price_to_level_id,
            price_levels,
            maker_side_peg_levels,
        ) = match taker_side {
            Side::Buy => (
                self.best_bid_price(),
                &mut self.limit.asks,
                &mut self.limit.levels,
                &mut self.pegged.ask_levels,
            ),
            Side::Sell => (
                self.best_ask_price(),
                &mut self.limit.bids,
                &mut self.limit.levels,
                &mut self.pegged.bid_levels,
            ),
        };

        let result = match_order(
            sequence_number,
            taker_side,
            taker_side_best_price,
            maker_side_price_to_level_id,
            price_levels,
            &mut self.limit.orders,
            maker_side_peg_levels,
            &mut self.pegged.orders,
            limit_price,
            quantity,
        );
        self.last_trade_price = result.last_trade_price();

        result
    }

    /// Computes the immediately executable quantity against the current book,
    /// capped by `requested_quantity`, without mutating state.
    ///
    /// Preconditions:
    /// - `requested_quantity` > 0
    /// - the book is not empty on the maker side
    ///
    /// Returns `requested_quantity` if fully executable; otherwise returns the
    /// available executable quantity.
    pub(crate) fn max_executable_quantity_unchecked(
        &self,
        taker_side: Side,
        requested_quantity: Quantity,
    ) -> Quantity {
        debug_assert!(!requested_quantity.is_zero());
        debug_assert!(!self.is_side_empty(taker_side.opposite()));

        let mut remaining = requested_quantity;

        // MidPrice peg level is active if the spread is less than or equal to 1
        let mid_active = self.spread().is_some_and(|spread| spread <= 1);

        match taker_side {
            Side::Buy => {
                // Iterate over the limit ask price levels
                for level_id in self.limit.asks.values() {
                    let level = &self.limit.levels[*level_id];
                    remaining = remaining.saturating_sub(level.total_quantity());
                    if remaining.is_zero() {
                        return requested_quantity;
                    }
                }
                // Primary peg level is always active
                remaining = remaining.saturating_sub(
                    self.pegged.ask_levels[PegReference::Primary.as_index()].quantity(),
                );
                if remaining.is_zero() {
                    return requested_quantity;
                }
                if mid_active {
                    remaining = remaining.saturating_sub(
                        self.pegged.ask_levels[PegReference::MidPrice.as_index()].quantity(),
                    );
                    if remaining.is_zero() {
                        return requested_quantity;
                    }
                }
            }
            Side::Sell => {
                // Iterate over the limit bid price levels
                for level_id in self.limit.bids.values().rev() {
                    let level = &self.limit.levels[*level_id];
                    remaining = remaining.saturating_sub(level.total_quantity());
                    if remaining.is_zero() {
                        return requested_quantity;
                    }
                }
                // Primary peg level is always active
                remaining = remaining.saturating_sub(
                    self.pegged.bid_levels[PegReference::Primary.as_index()].quantity(),
                );
                if mid_active {
                    remaining = remaining.saturating_sub(
                        self.pegged.bid_levels[PegReference::MidPrice.as_index()].quantity(),
                    );
                    if remaining.is_zero() {
                        return requested_quantity;
                    }
                }
            }
        }

        requested_quantity - remaining
    }

    /// Computes the immediately executable quantity with a limit price against the current book,
    /// capped by `requested_quantity`, without mutating state.
    ///
    /// Preconditions:
    /// - `requested_quantity` > 0
    /// - has crossable order at `limit_price`
    ///
    /// Returns `requested_quantity` if fully executable; otherwise returns the
    /// available executable quantity.
    pub(crate) fn max_executable_quantity_with_limit_price_unchecked(
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
                for (price, level_id) in self.limit.asks.iter() {
                    if *price > limit_price {
                        break;
                    }
                    let level = &self.limit.levels[*level_id];
                    remaining = remaining.saturating_sub(level.total_quantity());
                    if remaining.is_zero() {
                        return requested_quantity;
                    }
                }
                // Primary peg level is always active
                remaining = remaining.saturating_sub(
                    self.pegged.ask_levels[PegReference::Primary.as_index()].quantity(),
                );
                if remaining.is_zero() {
                    return requested_quantity;
                }
                if mid_active {
                    remaining = remaining.saturating_sub(
                        self.pegged.ask_levels[PegReference::MidPrice.as_index()].quantity(),
                    );
                    if remaining.is_zero() {
                        return requested_quantity;
                    }
                }
            }
            Side::Sell => {
                // Iterate over the limit bid price levels up to the limit price
                for (price, level_id) in self.limit.bids.iter().rev() {
                    if *price < limit_price {
                        break;
                    }
                    let level = &self.limit.levels[*level_id];
                    remaining = remaining.saturating_sub(level.total_quantity());
                    if remaining.is_zero() {
                        return requested_quantity;
                    }
                }
                // Primary peg level is always active
                remaining = remaining.saturating_sub(
                    self.pegged.bid_levels[PegReference::Primary.as_index()].quantity(),
                );
                if mid_active {
                    remaining = remaining.saturating_sub(
                        self.pegged.bid_levels[PegReference::MidPrice.as_index()].quantity(),
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
/// It decides the next order to consume by comparing the time priority of
/// the highest priority limit order and the highest priority pegged order.
/// If the time priority of them are the same, it consumes the limit order first.
///
/// Returns a `MatchResult` struct containing the result of the match.
#[allow(clippy::too_many_arguments)]
pub(crate) fn match_order(
    sequence_number: SequenceNumber,
    taker_side: Side,
    taker_side_best_price: Option<Price>,
    maker_side_price_to_level_id: &mut BTreeMap<Price, LevelId>,
    price_levels: &mut Slab<PriceLevel>,
    limit_orders: &mut FxHashMap<OrderId, RestingLimitOrder>,
    maker_side_peg_levels: &mut [PegLevel; PegReference::COUNT],
    pegged_orders: &mut FxHashMap<OrderId, RestingPeggedOrder>,
    limit_price: Option<Price>,
    quantity: Quantity,
) -> MatchResult {
    let mut match_result = MatchResult::new(taker_side);
    let mut remaining_quantity = quantity;
    let mut needs_reprice = false;

    while !remaining_quantity.is_zero() {
        let best_price_and_level_id = match taker_side {
            // The best price is the lowest price for a buy order (asks)
            Side::Buy => maker_side_price_to_level_id
                .iter()
                .next()
                .map(|(price, level_id)| (*price, *level_id)),
            // The best price is the highest price for a sell order (bids)
            Side::Sell => maker_side_price_to_level_id
                .iter()
                .next_back()
                .map(|(price, level_id)| (*price, *level_id)),
        };

        let Some((price, level_id)) = best_price_and_level_id else {
            break;
        };
        let price_level = &mut price_levels[level_id];

        if let Some(limit_price) = limit_price {
            match taker_side {
                Side::Buy if price > limit_price => break,
                Side::Sell if price < limit_price => break,
                _ => (),
            }
        }

        // Determine the active peg references based on the taker side best price
        // Primary: always active
        // Market: always inactive
        // MidPrice: active if the price is within 1 of the taker side best price
        let active_peg_references: &[PegReference] = match taker_side_best_price {
            Some(taker_side_best_price) if price.abs_diff(taker_side_best_price) <= 1 => {
                &MAKER_ARRAY_PRIMARY_MID_PRICE
            }
            _ => &MAKER_ARRAY_PRIMARY,
        };

        // Iterate over the orders at the best price levels
        while !remaining_quantity.is_zero() {
            let limit_queue_entry = {
                if price_level.is_empty() {
                    None
                } else {
                    // The price level is guaranteed to have at least one order
                    Some(price_level.peek().unwrap())
                }
            };
            let best_peg_info = get_best_peg_info(maker_side_peg_levels, active_peg_references);

            enum NextOrder {
                Limit(QueueEntry),
                Peg {
                    level_idx: usize,
                    queue_entry: QueueEntry,
                },
            }

            let next_order = match (limit_queue_entry, best_peg_info) {
                (
                    Some(limit_queue_entry),
                    Some((peg_level_idx, peg_time_priority, peg_queue_entry)),
                ) => {
                    if peg_time_priority < limit_queue_entry.time_priority() {
                        NextOrder::Peg {
                            level_idx: peg_level_idx,
                            queue_entry: peg_queue_entry,
                        }
                    } else {
                        NextOrder::Limit(limit_queue_entry)
                    }
                }
                (Some(limit_queue_entry), None) => NextOrder::Limit(limit_queue_entry),
                (None, Some((peg_level_idx, _peg_time_priority, peg_queue_entry))) => {
                    NextOrder::Peg {
                        level_idx: peg_level_idx,
                        queue_entry: peg_queue_entry,
                    }
                }
                (None, None) => {
                    break;
                }
            };

            match next_order {
                // Consume the limit order
                NextOrder::Limit(limit_queue_entry) => {
                    let order_id = limit_queue_entry.order_id();

                    let Some(order) = limit_orders.get_mut(&order_id) else {
                        // Stale queue entry in the price level, remove it
                        price_level.pop();
                        continue;
                    };
                    if limit_queue_entry.time_priority() != order.time_priority() {
                        // Stale queue entry in the price level, remove it
                        price_level.pop();
                        continue;
                    }

                    let (consumed, replenished) = order.match_against(remaining_quantity);
                    remaining_quantity -= consumed;
                    price_level.visible_quantity -= consumed;

                    match_result.add_trade(Trade::new(order_id, price, consumed));

                    if !replenished.is_zero() {
                        price_level.apply_replenishment(replenished);
                        price_level.reprioritize_front(sequence_number);
                        order.update_time_priority(sequence_number);
                    } else if order.is_filled() {
                        // The order is filled, remove it from the price level
                        price_level.remove_head_order(limit_orders);
                    }
                }
                // Consume the pegged order
                NextOrder::Peg {
                    level_idx,
                    queue_entry,
                } => {
                    let order_id = queue_entry.order_id();

                    let peg_level = &mut maker_side_peg_levels[level_idx];
                    let Some(order) = pegged_orders.get_mut(&order_id) else {
                        // Stale queue entry in the peg level, remove it
                        peg_level.pop();
                        continue;
                    };
                    if queue_entry.time_priority() != order.time_priority() {
                        // Stale queue entry in the peg level, remove it
                        peg_level.pop();
                        continue;
                    }

                    let consumed = order.match_against(remaining_quantity);
                    remaining_quantity -= consumed;
                    peg_level.quantity -= consumed;

                    match_result.add_trade(Trade::new(order_id, price, consumed));

                    // The order is filled, remove it from the peg level
                    if order.is_filled() {
                        peg_level.remove_head_order(pegged_orders);
                    }
                }
            }
        }

        if price_level.is_empty() {
            price_levels.remove(level_id);
            maker_side_price_to_level_id.remove(&price);
            needs_reprice = true;
        }
    }

    // Only the maker side primary peg reprice matters on the best price level removal
    if needs_reprice {
        maker_side_peg_levels[PegReference::Primary.as_index()].repriced_at = sequence_number;
    }

    match_result
}

/// Get the best (highest priority) pegged order information from the active peg references
///
/// The time priority is decided by the following criteria:
/// 1. When the level was last repriced
/// 2. When the order was entered into the level
///
/// Returns the tuple of (index of the peg level, time priority of the order, queue entry) if found; otherwise `None`.
fn get_best_peg_info(
    maker_side_peg_levels: &[PegLevel],
    active_peg_references: &[PegReference],
) -> Option<(usize, SequenceNumber, QueueEntry)> {
    let mut best: Option<(usize, SequenceNumber, QueueEntry)> = None;

    // Find the highest priority pegged order
    for peg_reference in active_peg_references {
        let idx = peg_reference.as_index();

        let peg_level = &maker_side_peg_levels[idx];
        let Some(queue_entry) = peg_level.peek() else {
            continue;
        };

        // If the order entered the peg level before the last reprice, the time priority should be adjusted
        let time_priority = max(peg_level.repriced_at(), queue_entry.time_priority());

        match best {
            None => best = Some((idx, time_priority, queue_entry)),
            Some((_, best_time_priority, best_queue_entry)) => {
                if time_priority < best_time_priority
                    || (time_priority == best_time_priority && queue_entry < best_queue_entry)
                {
                    best = Some((idx, time_priority, queue_entry));
                }
            }
        }
    }

    let (idx, time_priority, queue_entry) = best?;

    Some((idx, time_priority, queue_entry))
}

#[cfg(test)]
mod tests_match_order {
    use crate::*;
    use crate::{
        LimitOrder, Notional, OrderFlags, PeggedOrder, Quantity, QuantityPolicy, TimeInForce,
    };

    /// Helper function to create a new test order book
    fn new_test_book() -> OrderBook {
        OrderBook::new("TEST")
    }

    /// Helper function to add a standard limit order to the book
    fn add_standard_order(
        book: &mut OrderBook,
        sequence_number: SequenceNumber,
        id: OrderId,
        price: Price,
        quantity: Quantity,
        side: Side,
    ) {
        book.add_limit_order(
            sequence_number,
            id,
            LimitOrder::new(
                price,
                QuantityPolicy::Standard { quantity },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        );
    }

    /// Helper function to add an iceberg limit order to the book
    #[allow(clippy::too_many_arguments)]
    fn add_iceberg_order(
        book: &mut OrderBook,
        sequence_number: SequenceNumber,
        id: OrderId,
        price: Price,
        visible_quantity: Quantity,
        hidden_quantity: Quantity,
        replenish_quantity: Quantity,
        side: Side,
    ) {
        book.add_limit_order(
            sequence_number,
            id,
            LimitOrder::new(
                price,
                QuantityPolicy::Iceberg {
                    visible_quantity,
                    hidden_quantity,
                    replenish_quantity,
                },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        );
    }

    /// Helper function to add a pegged order to the book
    fn add_pegged_order(
        book: &mut OrderBook,
        sequence_number: SequenceNumber,
        id: OrderId,
        peg: PegReference,
        quantity: Quantity,
        side: Side,
    ) {
        book.add_pegged_order(
            sequence_number,
            id,
            PeggedOrder::new(
                peg,
                quantity,
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        );
    }

    #[test]
    fn test_single_maker_full_fill() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        let result =
            orderbook.match_order(SequenceNumber(1), Side::Buy, Some(Price(100)), Quantity(50));

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
        assert!(orderbook.best_ask_price().is_none());
    }

    #[test]
    fn test_single_maker_partial_fill() {
        let mut orderbook = new_test_book();
        assert!(orderbook.last_trade_price().is_none());
        assert!(orderbook.best_ask_price().is_none());

        // Add a sell order (maker) at 100 for 50
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );
        assert_eq!(orderbook.best_ask_price(), Some(Price(100)));

        // Match a buy order at 100 for 30 against the book
        let result =
            orderbook.match_order(SequenceNumber(1), Side::Buy, Some(Price(100)), Quantity(30));

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
        assert_eq!(orderbook.best_ask_price(), Some(Price(100)));

        // Match a buy order at 100 for 40 against the book
        let result =
            orderbook.match_order(SequenceNumber(2), Side::Buy, Some(Price(100)), Quantity(40));

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
        assert!(orderbook.best_ask_price().is_none());
    }

    #[test]
    fn test_sell_taker() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        let result = orderbook.match_order(
            SequenceNumber(1),
            Side::Sell,
            Some(Price(100)),
            Quantity(40),
        );

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
        assert_eq!(orderbook.best_bid_price(), Some(Price(100)));

        // Match a sell order at 100 for 20 against the book
        let result = orderbook.match_order(
            SequenceNumber(2),
            Side::Sell,
            Some(Price(100)),
            Quantity(20),
        );

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
        assert!(orderbook.best_bid_price().is_none());
    }

    #[test]
    fn test_empty_book_no_fill() {
        let mut orderbook = new_test_book();

        let result = orderbook.match_order(SequenceNumber(0), Side::Buy, None, Quantity(30));

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
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        // Buy limit 99 does not cross best ask 100
        let result =
            orderbook.match_order(SequenceNumber(1), Side::Buy, Some(Price(99)), Quantity(30));

        assert_eq!(result.executed_quantity(), Quantity(0));
        assert!(result.trades().is_empty());
        assert_eq!(orderbook.best_ask_price(), Some(Price(100)));
    }

    #[test]
    fn test_multiple_makers_same_price() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(20),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(100),
            Quantity(30),
            Side::Sell,
        );

        // Buy 40: fills first maker fully (20), second maker partially (20)
        let result =
            orderbook.match_order(SequenceNumber(2), Side::Buy, Some(Price(100)), Quantity(40));

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
        assert_eq!(orderbook.best_ask_price(), Some(Price(100)));
    }

    #[test]
    fn test_multiple_price_levels() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(30),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(101),
            Quantity(40),
            Side::Sell,
        );

        // Buy 50 at limit 101: 30 @ 100, then 20 @ 101
        let result =
            orderbook.match_order(SequenceNumber(2), Side::Buy, Some(Price(101)), Quantity(50));

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
        assert_eq!(orderbook.best_ask_price(), Some(Price(101)));
    }

    #[test]
    fn test_market_buy_sweeps_levels() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(25),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(101),
            Quantity(25),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(2),
            OrderId(2),
            Price(102),
            Quantity(25),
            Side::Sell,
        );

        // Market buy (None limit) for 60: 25 @ 100, 25 @ 101, 10 @ 102
        let result = orderbook.match_order(SequenceNumber(3), Side::Buy, None, Quantity(60));

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
        assert_eq!(orderbook.best_ask_price(), Some(Price(102)));
    }

    // --- Iceberg test cases ---

    #[test]
    fn test_iceberg_maker_partial_fill_visible_only() {
        let mut orderbook = new_test_book();
        // Iceberg: visible 20, hidden 30, replenish 10 (total 50)
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(20),
            Quantity(30),
            Quantity(10),
            Side::Sell,
        );

        // Buy 15: only consumes visible, no replenish yet
        let result =
            orderbook.match_order(SequenceNumber(1), Side::Buy, Some(Price(100)), Quantity(15));

        assert_eq!(result.executed_quantity(), Quantity(15));
        assert_eq!(result.executed_value(), Notional(100 * 15));
        assert_eq!(result.trades().len(), 1);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(15))
        );
        assert_eq!(orderbook.last_trade_price(), Some(Price(100)));
        // Iceberg still has 5 visible + 30 hidden at 100
        assert_eq!(orderbook.best_ask_price(), Some(Price(100)));
    }

    #[test]
    fn test_iceberg_maker_replenish_during_match() {
        let mut orderbook = new_test_book();
        // Iceberg: visible 10, hidden 20, replenish 10
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );

        // Buy 15: consumes 10 (trade 10), replenish 10, then consumes 5 (trade 5)
        let result =
            orderbook.match_order(SequenceNumber(1), Side::Buy, Some(Price(100)), Quantity(15));

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
        assert_eq!(orderbook.best_ask_price(), Some(Price(100)));
    }

    #[test]
    fn test_iceberg_maker_multiple_replenishes_in_one_match() {
        let mut orderbook = new_test_book();
        // Iceberg: visible 10, hidden 30, replenish 10 (total 40)
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(30),
            Quantity(10),
            Side::Sell,
        );

        // Buy 35: 10 + 10 (replenish) + 10 + 5
        let result =
            orderbook.match_order(SequenceNumber(1), Side::Buy, Some(Price(100)), Quantity(35));

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
        assert_eq!(orderbook.best_ask_price(), Some(Price(100)));
    }

    #[test]
    fn test_iceberg_maker_fully_filled() {
        let mut orderbook = new_test_book();
        // Iceberg: visible 10, hidden 20, replenish 10 (total 30)
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );

        let result =
            orderbook.match_order(SequenceNumber(1), Side::Buy, Some(Price(100)), Quantity(30));

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
        assert!(orderbook.best_ask_price().is_none());
    }

    #[test]
    fn test_iceberg_then_standard_same_price() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        // Buy 70: replenish moves iceberg to back, so we get 10 (iceberg), then 50 (standard), then 10 (iceberg) = 3 trades
        let result =
            orderbook.match_order(SequenceNumber(2), Side::Buy, Some(Price(100)), Quantity(70));

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
        assert_eq!(orderbook.best_ask_price(), Some(Price(100)));
    }

    #[test]
    fn test_iceberg_sell_taker_against_bids() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Buy,
        );

        // Sell 25: 10 + 10 (replenish) + 5
        let result = orderbook.match_order(
            SequenceNumber(1),
            Side::Sell,
            Some(Price(100)),
            Quantity(25),
        );

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
        assert_eq!(orderbook.best_bid_price(), Some(Price(100)));
    }

    #[test]
    fn test_limit_order_prioritized_when_older_than_pegged() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Side::Sell,
        );
        add_pegged_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            PegReference::Primary,
            Quantity(10),
            Side::Sell,
        );

        let result =
            orderbook.match_order(SequenceNumber(2), Side::Buy, Some(Price(100)), Quantity(15));

        assert_eq!(result.executed_quantity(), Quantity(15));
        assert_eq!(result.trades().len(), 2);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(1), Price(100), Quantity(5))
        );
    }

    #[test]
    fn test_pegged_order_prioritized_when_older_than_limit() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Side::Sell,
        );
        add_pegged_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            PegReference::Primary,
            Quantity(10),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(2),
            OrderId(2),
            Price(100),
            Quantity(10),
            Side::Sell,
        );

        let result =
            orderbook.match_order(SequenceNumber(3), Side::Buy, Some(Price(100)), Quantity(25));

        assert_eq!(result.executed_quantity(), Quantity(25));
        assert_eq!(result.trades().len(), 3);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(0), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(1), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[2],
            Trade::new(OrderId(2), Price(100), Quantity(5))
        );
    }

    #[test]
    fn test_limit_order_prioritized_when_time_priority_ties() {
        let mut orderbook = new_test_book();
        add_pegged_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            PegReference::Primary,
            Quantity(10),
            Side::Sell,
        );
        // The primary peg level is repriced at sequence number 1 when the standard order is added
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(100),
            Quantity(10),
            Side::Sell,
        );

        let result =
            orderbook.match_order(SequenceNumber(2), Side::Buy, Some(Price(100)), Quantity(15));

        assert_eq!(result.executed_quantity(), Quantity(15));
        assert_eq!(result.trades().len(), 2);
        assert_eq!(
            result.trades()[0],
            Trade::new(OrderId(1), Price(100), Quantity(10))
        );
        assert_eq!(
            result.trades()[1],
            Trade::new(OrderId(0), Price(100), Quantity(5))
        );
    }
}

#[cfg(test)]
mod tests_max_executable_quantity_unchecked {
    use crate::*;
    use crate::{LimitOrder, OrderFlags, PeggedOrder, Quantity, QuantityPolicy, TimeInForce};

    // Helper function to create a new test order book
    fn new_test_book() -> OrderBook {
        OrderBook::new("TEST")
    }

    // Helper function to add a standard limit order to the book
    fn add_standard_order(
        book: &mut OrderBook,
        id: OrderId,
        sequence_number: SequenceNumber,
        price: Price,
        quantity: Quantity,
        side: Side,
    ) {
        book.add_limit_order(
            sequence_number,
            id,
            LimitOrder::new(
                price,
                QuantityPolicy::Standard { quantity },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        );
    }

    // Helper function to add a pegged order to the book
    fn add_pegged_order(
        book: &mut OrderBook,
        id: OrderId,
        sequence_number: SequenceNumber,
        peg: PegReference,
        quantity: Quantity,
        side: Side,
    ) {
        book.add_pegged_order(
            sequence_number,
            id,
            PeggedOrder::new(
                peg,
                quantity,
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        );
    }

    #[test]
    fn test_buy_fully_executable_returns_requested() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        let qty = book.max_executable_quantity_unchecked(Side::Buy, Quantity(30));
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_buy_capped_by_available_liquidity() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        let qty = book.max_executable_quantity_unchecked(Side::Buy, Quantity(100));
        assert_eq!(qty, Quantity(50));
    }

    #[test]
    fn test_buy_multiple_limit_levels_summed() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(100),
            Quantity(30),
            Side::Sell,
        );
        add_standard_order(
            &mut book,
            OrderId(1),
            SequenceNumber(1),
            Price(101),
            Quantity(40),
            Side::Sell,
        );

        // All ask levels: 30 + 40 = 70. Request 100 → 70 executable.
        let qty = book.max_executable_quantity_unchecked(Side::Buy, Quantity(100));
        assert_eq!(qty, Quantity(70));
    }

    #[test]
    fn test_sell_fully_executable_returns_requested() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        let qty = book.max_executable_quantity_unchecked(Side::Sell, Quantity(30));
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_sell_capped_by_available_liquidity() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        let qty = book.max_executable_quantity_unchecked(Side::Sell, Quantity(100));
        assert_eq!(qty, Quantity(50));
    }

    #[test]
    fn test_sell_multiple_limit_levels_summed() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(100),
            Quantity(30),
            Side::Buy,
        );
        add_standard_order(
            &mut book,
            OrderId(1),
            SequenceNumber(1),
            Price(99),
            Quantity(40),
            Side::Buy,
        );

        // All bid levels (best first): 30 + 40 = 70. Request 100 → 70 executable.
        let qty = book.max_executable_quantity_unchecked(Side::Sell, Quantity(100));
        assert_eq!(qty, Quantity(70));
    }

    #[test]
    fn test_buy_includes_primary_peg_ask() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(100),
            Quantity(20),
            Side::Sell,
        );
        add_pegged_order(
            &mut book,
            OrderId(1),
            SequenceNumber(1),
            PegReference::Primary,
            Quantity(15),
            Side::Sell,
        );

        // Limit asks 20 + primary peg 15 = 35. Request 50 → 35 executable.
        let qty = book.max_executable_quantity_unchecked(Side::Buy, Quantity(50));
        assert_eq!(qty, Quantity(35));
    }

    #[test]
    fn test_sell_includes_primary_peg_bid() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(100),
            Quantity(20),
            Side::Buy,
        );
        add_pegged_order(
            &mut book,
            OrderId(1),
            SequenceNumber(1),
            PegReference::Primary,
            Quantity(15),
            Side::Buy,
        );

        // Limit bids 20 + primary peg 15 = 35. Request 50 → 35 executable.
        let qty = book.max_executable_quantity_unchecked(Side::Sell, Quantity(50));
        assert_eq!(qty, Quantity(35));
    }

    #[test]
    fn test_buy_includes_mid_price_peg_when_spread_at_most_one() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(100),
            Quantity(10),
            Side::Buy,
        );
        add_standard_order(
            &mut book,
            OrderId(1),
            SequenceNumber(1),
            Price(101),
            Quantity(10),
            Side::Sell,
        );
        add_pegged_order(
            &mut book,
            OrderId(2),
            SequenceNumber(2),
            PegReference::Primary,
            Quantity(5),
            Side::Sell,
        );
        add_pegged_order(
            &mut book,
            OrderId(3),
            SequenceNumber(3),
            PegReference::MidPrice,
            Quantity(7),
            Side::Sell,
        );

        // Spread = 0 (bid 100, ask 100), so mid_active. Limit asks 10 + primary 5 + mid 7 = 22.
        let qty = book.max_executable_quantity_unchecked(Side::Buy, Quantity(100));
        assert_eq!(qty, Quantity(22));
    }

    #[test]
    fn test_buy_excludes_mid_price_peg_when_spread_wide() {
        let mut book = new_test_book();
        add_standard_order(
            &mut book,
            OrderId(0),
            SequenceNumber(0),
            Price(98),
            Quantity(10),
            Side::Buy,
        );
        add_standard_order(
            &mut book,
            OrderId(1),
            SequenceNumber(1),
            Price(102),
            Quantity(10),
            Side::Sell,
        );
        add_pegged_order(
            &mut book,
            OrderId(2),
            SequenceNumber(2),
            PegReference::Primary,
            Quantity(5),
            Side::Sell,
        );
        add_pegged_order(
            &mut book,
            OrderId(3),
            SequenceNumber(3),
            PegReference::MidPrice,
            Quantity(7),
            Side::Sell,
        );

        // Spread = 4 > 1, so mid_active = false. Only limit asks 10 + primary 5 = 15.
        let qty = book.max_executable_quantity_unchecked(Side::Buy, Quantity(100));
        assert_eq!(qty, Quantity(15));
    }
}

#[cfg(test)]
mod tests_max_executable_quantity_with_limit_price_unchecked {
    use crate::*;
    use crate::{LimitOrder, OrderFlags, Quantity, QuantityPolicy, TimeInForce};

    /// Helper function to create a new test order book
    fn new_test_book() -> OrderBook {
        OrderBook::new("TEST")
    }

    /// Helper function to add a standard limit order to the book
    fn add_standard_order(
        book: &mut OrderBook,
        sequence_number: SequenceNumber,
        id: OrderId,
        price: Price,
        quantity: Quantity,
        side: Side,
    ) {
        book.add_limit_order(
            sequence_number,
            id,
            LimitOrder::new(
                price,
                QuantityPolicy::Standard { quantity },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        );
    }

    /// Helper function to add an iceberg limit order to the book
    #[allow(clippy::too_many_arguments)]
    fn add_iceberg_order(
        book: &mut OrderBook,
        sequence_number: SequenceNumber,
        id: OrderId,
        price: Price,
        visible_quantity: Quantity,
        hidden_quantity: Quantity,
        replenish_quantity: Quantity,
        side: Side,
    ) {
        book.add_limit_order(
            sequence_number,
            id,
            LimitOrder::new(
                price,
                QuantityPolicy::Iceberg {
                    visible_quantity,
                    hidden_quantity,
                    replenish_quantity,
                },
                OrderFlags::new(side, false, TimeInForce::Gtc),
            ),
        );
    }

    #[test]
    fn test_fully_executable_returns_requested() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        // Buy 30 at 100: 30 available, request 30 → fully executable
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Buy,
            Price(100),
            Quantity(30),
        );
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_capped_by_available_liquidity() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Sell,
        );

        // Buy 100 at 100: only 50 available
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Buy,
            Price(100),
            Quantity(100),
        );
        assert_eq!(qty, Quantity(50));
    }

    #[test]
    fn test_multiple_levels_summed_up_to_limit_price() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(30),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(101),
            Quantity(40),
            Side::Sell,
        );

        // Buy at limit 101: 30 + 40 = 70 available, request 100 → 70 executable
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Buy,
            Price(101),
            Quantity(100),
        );
        assert_eq!(qty, Quantity(70));
    }

    #[test]
    fn test_buy_respects_limit_price_ceiling() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(102),
            Quantity(20),
            Side::Sell,
        );

        // Buy at limit 101: only 10 @ 100 counts, 102 is above limit
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Buy,
            Price(101),
            Quantity(100),
        );
        assert_eq!(qty, Quantity(10));
    }

    #[test]
    fn test_sell_taker_fully_executable() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        // Sell 30 at 100: 30 executable
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Sell,
            Price(100),
            Quantity(30),
        );
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_sell_taker_capped_by_bids() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        // Sell 100 at 100: only 50 available
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Sell,
            Price(100),
            Quantity(100),
        );
        assert_eq!(qty, Quantity(50));
    }

    #[test]
    fn test_sell_respects_limit_price_floor() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(98),
            Quantity(30),
            Side::Buy,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(100),
            Quantity(50),
            Side::Buy,
        );

        // Sell at limit 99: only 50 @ 100 counts (bid >= 99), 98 is below limit
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Sell,
            Price(99),
            Quantity(100),
        );
        assert_eq!(qty, Quantity(50));
    }

    // --- Iceberg test cases (total = visible + hidden at each level) ---

    #[test]
    fn test_iceberg_buy_capped_by_total_quantity() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );

        // Buy at 100: executable = visible + hidden = 30, request 50 → 30
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Buy,
            Price(100),
            Quantity(50),
        );
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_iceberg_buy_fully_executable() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(25),
            Quantity(10),
            Side::Sell,
        );

        // Buy at 100: 35 total available, request 20 → 20 (fully executable)
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Buy,
            Price(100),
            Quantity(20),
        );
        assert_eq!(qty, Quantity(20));
    }

    #[test]
    fn test_iceberg_sell_capped_by_total_quantity() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(15),
            Quantity(25),
            Quantity(10),
            Side::Buy,
        );

        // Sell at 100: executable = 40 total, request 50 → 40
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Sell,
            Price(100),
            Quantity(50),
        );
        assert_eq!(qty, Quantity(40));
    }

    #[test]
    fn test_iceberg_and_standard_levels_summed() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(101),
            Quantity(40),
            Side::Sell,
        );

        // Buy at 101: 30 (iceberg) + 40 (standard) = 70, request 100 → 70
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Buy,
            Price(101),
            Quantity(100),
        );
        assert_eq!(qty, Quantity(70));
    }

    #[test]
    fn test_iceberg_respects_buy_limit_price_ceiling() {
        let mut orderbook = new_test_book();
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(100),
            Quantity(10),
            Quantity(20),
            Quantity(10),
            Side::Sell,
        );
        add_standard_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(102),
            Quantity(50),
            Side::Sell,
        );

        // Buy at limit 101: only 30 @ 100 counts, 102 is above limit
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Buy,
            Price(101),
            Quantity(100),
        );
        assert_eq!(qty, Quantity(30));
    }

    #[test]
    fn test_iceberg_respects_sell_limit_price_floor() {
        let mut orderbook = new_test_book();
        add_standard_order(
            &mut orderbook,
            SequenceNumber(0),
            OrderId(0),
            Price(98),
            Quantity(30),
            Side::Buy,
        );
        add_iceberg_order(
            &mut orderbook,
            SequenceNumber(1),
            OrderId(1),
            Price(100),
            Quantity(10),
            Quantity(40),
            Quantity(10),
            Side::Buy,
        );

        // Sell at limit 99: only 50 @ 100 (iceberg total) counts, 98 is below limit
        let qty = orderbook.max_executable_quantity_with_limit_price_unchecked(
            Side::Sell,
            Price(99),
            Quantity(100),
        );
        assert_eq!(qty, Quantity(50));
    }
}
