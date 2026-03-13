mod helpers;

use matchcore::*;

fn main() {
    let mut book = OrderBook::new("ETH/USD");

    // Submit standard buy orders from the best price to the worst price
    for i in 0..10 {
        book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(100 - i),
                    QuantityPolicy::Standard {
                        quantity: Quantity(100),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
    }

    // Submit standard sell orders from the best price to the worst price
    for i in 0..10 {
        book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(110 + i),
                    QuantityPolicy::Standard {
                        quantity: Quantity(100),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
    }

    let best_bid = book.best_bid().unwrap();
    println!("Best bid: {}@{}", best_bid.1, best_bid.0);
    let best_ask = book.best_ask().unwrap();
    println!("Best ask: {}@{}", best_ask.1, best_ask.0);

    println!("Best bid price: {}", book.best_bid_price().unwrap());
    println!("Best ask price: {}", book.best_ask_price().unwrap());

    println!("Best bid volume: {}", book.best_bid_volume().unwrap());
    println!("Best ask volume: {}", book.best_ask_volume().unwrap());

    println!("Is side empty: {}", book.is_side_empty(Side::Buy));
    println!("Is side empty: {}", book.is_side_empty(Side::Sell));

    println!(
        "Has crossable order: {}",
        book.has_crossable_order(Side::Buy, Price(100))
    );
    println!(
        "Has crossable order: {}",
        book.has_crossable_order(Side::Sell, Price(100))
    );

    println!("Spread: {}", book.spread().unwrap());
    println!("Mid price: {}", book.mid_price().unwrap());
    println!("Micro price: {}", book.micro_price().unwrap());

    println!("Bid volume: {}", book.bid_volume(10));
    println!("Ask volume: {}", book.ask_volume(10));

    println!("Is thin book: {}", book.is_thin_book(Quantity(100), 10));
    println!("Order book imbalance: {}", book.order_book_imbalance(10));

    let bid_depth_stats = book.depth_statistics(Side::Buy, 10);
    println!(
        "Bid depth stats: analyzed levels: {}, total value: {}, total volume: {}, average level size: {}, min level size: {}, max level size: {}, std dev level size: {}, vwap: {}",
        bid_depth_stats.n_analyzed_levels(),
        bid_depth_stats.total_value(),
        bid_depth_stats.total_volume(),
        bid_depth_stats.average_level_size(),
        bid_depth_stats.min_level_size(),
        bid_depth_stats.max_level_size(),
        bid_depth_stats.std_dev_level_size(),
        bid_depth_stats.vwap()
    );
    let ask_depth_stats = book.depth_statistics(Side::Sell, 10);
    println!(
        "Ask depth stats: analyzed levels: {}, total value: {}, total volume: {}, average level size: {}, min level size: {}, max level size: {}, std dev level size: {}, vwap: {}",
        ask_depth_stats.n_analyzed_levels(),
        ask_depth_stats.total_value(),
        ask_depth_stats.total_volume(),
        ask_depth_stats.average_level_size(),
        ask_depth_stats.min_level_size(),
        ask_depth_stats.max_level_size(),
        ask_depth_stats.std_dev_level_size(),
        ask_depth_stats.vwap()
    );

    let (buy_pressure, sell_pressure) = book.buy_sell_pressure();
    println!(
        "Buy sell pressure: buy pressure: {}, sell pressure: {}",
        buy_pressure, sell_pressure
    );

    println!(
        "Price at depth: {}",
        book.price_at_depth(Side::Buy, Quantity(500)).unwrap()
    );
    println!(
        "Price at depth: {}",
        book.price_at_depth(Side::Sell, Quantity(500)).unwrap()
    );

    println!("VWAP: {}", book.vwap(Side::Buy, Quantity(500)).unwrap());
    println!("VWAP: {}", book.vwap(Side::Sell, Quantity(500)).unwrap());

    let buy_market_impact = book.market_impact(Side::Buy, Quantity(500));
    println!(
        "Market impact: requested quantity: {}, available quantity: {}, total cost: {}, best price: {}, worst price: {}, consumed price levels: {}, average price: {}, slippage: {}",
        buy_market_impact.requested_quantity(),
        buy_market_impact.available_quantity(),
        buy_market_impact.total_cost(),
        buy_market_impact.best_price(),
        buy_market_impact.worst_price(),
        buy_market_impact.consumed_price_levels(),
        buy_market_impact.average_price(),
        buy_market_impact.slippage()
    );
    let sell_market_impact = book.market_impact(Side::Sell, Quantity(500));
    println!(
        "Market impact: requested quantity: {}, available quantity: {}, total cost: {}, best price: {}, worst price: {}, consumed price levels: {}, average price: {}, slippage: {}",
        sell_market_impact.requested_quantity(),
        sell_market_impact.available_quantity(),
        sell_market_impact.total_cost(),
        sell_market_impact.best_price(),
        sell_market_impact.worst_price(),
        sell_market_impact.consumed_price_levels(),
        sell_market_impact.average_price(),
        sell_market_impact.slippage()
    );
}
