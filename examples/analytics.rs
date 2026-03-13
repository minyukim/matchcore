//! Example: run analytics on a populated order book
//!
//! Run: cargo run --example analytics

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
    println!("Best bid: {} x {}", best_bid.0, best_bid.1);
    let best_ask = book.best_ask().unwrap();
    println!("Best ask: {} x {}", best_ask.0, best_ask.1);

    println!();

    println!("Best bid price: {}", book.best_bid_price().unwrap());
    println!("Best ask price: {}", book.best_ask_price().unwrap());

    println!();

    println!("Best bid size: {}", book.best_bid_size().unwrap());
    println!("Best ask size: {}", book.best_ask_size().unwrap());

    println!();

    println!("Is bid side empty: {}", book.is_side_empty(Side::Buy));
    println!("Is ask side empty: {}", book.is_side_empty(Side::Sell));

    println!();

    println!(
        "Has crossable ask order: {}",
        book.has_crossable_order(Side::Buy, Price(100))
    );
    println!(
        "Has crossable bid order: {}",
        book.has_crossable_order(Side::Sell, Price(100))
    );

    println!();

    println!("Spread: {}", book.spread().unwrap());
    println!("Mid price: {}", book.mid_price().unwrap());
    println!("Micro price: {}", book.micro_price().unwrap());

    println!();

    println!("Bid size: {}", book.bid_size(10));
    println!("Ask size: {}", book.ask_size(10));

    println!();

    println!("Is thin book: {}", book.is_thin_book(Quantity(100), 10));
    println!("Order book imbalance: {}", book.order_book_imbalance(10));

    println!();

    let bid_depth_stats = book.depth_statistics(Side::Buy, 10);
    println!(
        "Bid depth stats: analyzed levels: {}, total value: {}, total size: {}, average level size: {}, min level size: {}, max level size: {}, std dev level size: {}, vwap: {}",
        bid_depth_stats.n_analyzed_levels(),
        bid_depth_stats.total_value(),
        bid_depth_stats.total_size(),
        bid_depth_stats.average_level_size(),
        bid_depth_stats.min_level_size(),
        bid_depth_stats.max_level_size(),
        bid_depth_stats.std_dev_level_size(),
        bid_depth_stats.vwap()
    );
    let ask_depth_stats = book.depth_statistics(Side::Sell, 10);
    println!(
        "Ask depth stats: analyzed levels: {}, total value: {}, total size: {}, average level size: {}, min level size: {}, max level size: {}, std dev level size: {}, vwap: {}",
        ask_depth_stats.n_analyzed_levels(),
        ask_depth_stats.total_value(),
        ask_depth_stats.total_size(),
        ask_depth_stats.average_level_size(),
        ask_depth_stats.min_level_size(),
        ask_depth_stats.max_level_size(),
        ask_depth_stats.std_dev_level_size(),
        ask_depth_stats.vwap()
    );

    println!();

    let (buy_pressure, sell_pressure) = book.buy_sell_pressure();
    println!(
        "Buy sell pressure: buy {}, sell {}",
        buy_pressure, sell_pressure
    );

    println!();

    println!(
        "Bid price at depth: {}",
        book.price_at_depth(Side::Buy, Quantity(500)).unwrap()
    );
    println!(
        "Ask price at depth: {}",
        book.price_at_depth(Side::Sell, Quantity(500)).unwrap()
    );

    println!();

    println!("Buy VWAP: {}", book.vwap(Side::Buy, Quantity(500)).unwrap());
    println!(
        "Sell VWAP: {}",
        book.vwap(Side::Sell, Quantity(500)).unwrap()
    );

    println!();

    let buy_market_impact = book.market_impact(Side::Buy, Quantity(500));
    println!(
        "Buy market impact: requested quantity: {}, available quantity: {}, total cost: {}, best price: {}, worst price: {}, consumed price levels: {}, average price: {}, slippage: {}",
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
        "Sell market impact: requested quantity: {}, available quantity: {}, total cost: {}, best price: {}, worst price: {}, consumed price levels: {}, average price: {}, slippage: {}",
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
