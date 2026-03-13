mod helpers;

use matchcore::*;

fn main() {
    let mut book = OrderBook::new("ETH/USD");

    // Submit a market buy order
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

    // Submit a market sell order
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

    // Submit standard buy orders from the best price to the worst price
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

    // Submit standard sell orders from the best price to the worst price
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

    // Submit market buy orders
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

    // Submit market sell orders
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
