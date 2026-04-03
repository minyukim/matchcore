//! Example: submit market orders against a resting book
//!
//! Flow:
//! 1. Submit a market buy and a market sell with no liquidity (cancelled right away).
//! 2. Stack bids (100 down to 91) and asks (110 up to 119) with standard limits.
//! 3. Send several market buys and market sells that consume multiple price levels.
//!
//! Run: `cargo run --example market_orders`

mod helpers;

use matchcore::*;

fn main() {
    let mut book = OrderBook::new("ETH/USD");

    println!("=== Submit a market bid @ 200 (no liquidity -> cancelled) ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Market(MarketOrder::new(Quantity(200), Side::Buy, false)),
        }),
    });
    println!("{}", outcome);

    println!("=== Submit a market ask @ 200 (no liquidity -> cancelled) ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Market(MarketOrder::new(Quantity(200), Side::Sell, false)),
        }),
    });
    println!("{}", outcome);

    println!("=== Submit bids stepping down from 100 ===");
    for i in 0..10 {
        let outcome = book.execute(&Command {
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
        println!("{}", outcome);
    }

    println!("=== Submit asks stepping up from 110 ===");
    for i in 0..10 {
        let outcome = book.execute(&Command {
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
        println!("{}", outcome);
    }

    println!("=== Submit market bids  ===");
    for _ in 0..5 {
        let outcome = book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(210), Side::Buy, false)),
            }),
        });
        println!("{}", outcome);
    }

    println!("=== Submit market asks ===");
    for _ in 0..5 {
        let outcome = book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(210), Side::Sell, false)),
            }),
        });
        println!("{}", outcome);
    }
}
