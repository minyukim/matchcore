use crate::{
    MatchResult, PegReference, Side, Trade,
    orderbook::{
        OrderBook,
        peg_level::{MAKER_ARRAY_PRIMARY, MAKER_ARRAY_PRIMARY_MID_PRICE},
    },
};

impl OrderBook {
    /// Match an order against existing orders in the order book.
    ///
    /// It first tries to find the best price level for the order,
    /// and match against the orders in the price level.
    /// If the price level is exhausted, it starts consuming the active peg levels.
    /// If the active peg levels are exhausted, it moves to the next best price level,
    /// and then active peg levels again, and so on.
    /// It stops when the order is fully matched or all the levels are exhausted.
    ///
    /// Returns a `MatchResult` struct containing the result of the match.
    #[allow(unused)]
    pub(super) fn match_order(
        &mut self,
        taker_side: Side,
        price_limit: Option<u64>,
        quantity: u64,
        timestamp: u64,
    ) -> MatchResult {
        let mut match_result = MatchResult::new(taker_side);
        let mut remaining_quantity = quantity;

        let taker_side_best_price = match taker_side {
            Side::Buy => self.best_bid(),
            Side::Sell => self.best_ask(),
        };

        let maker_side_price_levels = match taker_side {
            Side::Buy => &mut self.limit_ask_levels,
            Side::Sell => &mut self.limit_bid_levels,
        };

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

            if let Some(price_limit) = price_limit {
                match taker_side {
                    Side::Buy if price > price_limit => break,
                    Side::Sell if price < price_limit => break,
                    _ => (),
                }
            }

            // The price level is guaranteed to exist because the best price is not None
            let price_level = maker_side_price_levels.get_mut(&price).unwrap();

            // Iterate over the orders at the price level
            while remaining_quantity > 0 {
                // The price level is guaranteed to have at least one order
                let order = price_level.peek(&mut self.limit_orders).unwrap();
                let order_id = order.id();

                if order.is_expired(timestamp) {
                    // Remove the order if it is expired
                    price_level.remove_head_order(&mut self.limit_orders);
                    if price_level.is_empty() {
                        maker_side_price_levels.remove(&price);
                        break;
                    }

                    continue;
                }

                let (consumed, replenished) = order.match_against(remaining_quantity);
                remaining_quantity -= consumed;

                price_level.visible_quantity -= consumed;
                price_level.handle_replenishment(replenished);

                match_result.add_trade(Trade::new(order_id, price, consumed));
                self.last_trade_price = Some(price);

                if order.is_filled() {
                    // Remove the order if it is filled
                    price_level.remove_head_order(&mut self.limit_orders);
                    if price_level.is_empty() {
                        maker_side_price_levels.remove(&price);
                        break;
                    }
                }
            }

            let maker_side_peg_levels = match taker_side {
                Side::Buy => &mut self.peg_ask_levels,
                Side::Sell => &mut self.peg_bid_levels,
            };

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

                for peg_reference in active_peg_references {
                    let idx = peg_reference.as_index();

                    let candidate_id = {
                        let peg_level = &mut maker_side_peg_levels[idx];
                        peg_level.peek_order_id(&self.pegged_orders)
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

                let Some((best_level_idx, order_id)) = best else {
                    break;
                };

                let peg_level = &mut maker_side_peg_levels[best_level_idx];

                // The order is guaranteed to exist because the order ID is found in the peg level
                let order = self.pegged_orders.get_mut(&order_id).unwrap();

                if order.is_expired(timestamp) {
                    peg_level.remove_head_order(&mut self.pegged_orders);
                    continue;
                }

                let consumed = order.match_against(remaining_quantity);
                remaining_quantity -= consumed;

                peg_level.quantity -= consumed;

                match_result.add_trade(Trade::new(order_id, price, consumed));
                self.last_trade_price = Some(price);

                if order.is_filled() {
                    peg_level.remove_head_order(&mut self.pegged_orders);
                }
            }
        }

        match_result
    }
}
