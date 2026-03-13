//! Example: submit/amend/cancel standard orders
//!
//! Run: cargo run --example standard_orders

mod helpers;

use matchcore::*;

fn main() {
    let mut book = OrderBook::new("ETH/USD");

    // Submit a standard buy order
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);

    let target_order_id = helpers::target_order_id(&outcome).unwrap();

    // Amend the buy order
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Amend(AmendCmd {
            order_id: target_order_id,
            patch: AmendPatch::Limit(
                LimitOrderPatch::new()
                    .with_price(Price(101))
                    .with_quantity_policy(QuantityPolicy::Standard {
                        quantity: Quantity(20),
                    }),
            ),
        }),
    });
    println!("{}", outcome);

    let new_target_order_id = helpers::target_order_id(&outcome).unwrap();

    // Submit a standard marketable sell order
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(101),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);

    // Cancel the remaining buy order
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Cancel(CancelCmd {
            order_id: new_target_order_id,
            order_kind: OrderKind::Limit,
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

    // Submit standard marketable buy orders
    for _ in 0..5 {
        let outcome = book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(120),
                    QuantityPolicy::Standard {
                        quantity: Quantity(200),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        println!("{}", outcome);
    }

    // Submit standard marketable sell orders
    for _ in 0..5 {
        let outcome = book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(90),
                    QuantityPolicy::Standard {
                        quantity: Quantity(200),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        println!("{}", outcome);
    }
}
