//! Example: retrieve level 1 and level 2 market data on a populated order book
//!
//! Run: cargo run --example market_data

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

    let l1 = Level1::from(&book);
    println!("#################### Level 1 ####################");
    println!("{}", l1);

    println!("Spread: {}", l1.spread().unwrap());
    println!("Mid price: {}", l1.mid_price().unwrap());
    println!("Micro price: {}", l1.micro_price().unwrap());

    println!();

    let l2: Level2 = Level2::from(&book);
    println!("#################### Level 2 ####################");
    println!("{}", l2);

    println!("Spread: {}", l2.spread().unwrap());
    println!("Mid price: {}", l2.mid_price().unwrap());
    println!("Micro price: {}", l2.micro_price().unwrap());

    println!();

    println!("Bid size: {}", l2.bid_size(10));
    println!("Ask size: {}", l2.ask_size(10));

    println!();

    println!("Is thin book: {}", l2.is_thin_book(Quantity(100), 10));
    println!("Order book imbalance: {}", l2.order_book_imbalance(10));

    println!();

    let bid_depth_stats = l2.depth_statistics(Side::Buy, 10);
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
    let ask_depth_stats = l2.depth_statistics(Side::Sell, 10);
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

    println!(
        "Bid price at depth: {}",
        l2.price_at_depth(Side::Buy, Quantity(500)).unwrap()
    );
    println!(
        "Ask price at depth: {}",
        l2.price_at_depth(Side::Sell, Quantity(500)).unwrap()
    );

    println!();

    println!("Buy VWAP: {}", l2.vwap(Side::Buy, Quantity(500)).unwrap());
    println!("Sell VWAP: {}", l2.vwap(Side::Sell, Quantity(500)).unwrap());
}
