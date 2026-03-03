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
