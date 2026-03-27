//! Example: submit/amend/cancel iceberg orders
//!
//! Run: cargo run --example iceberg_orders

mod helpers;

use matchcore::*;

fn main() {
    let mut book = OrderBook::new("ETH/USD");

    // Submit an iceberg buy order
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(100),
                QuantityPolicy::Iceberg {
                    visible_quantity: Quantity(10),
                    hidden_quantity: Quantity(90),
                    replenish_quantity: Quantity(10),
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
                    .with_quantity_policy(QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(20),
                        hidden_quantity: Quantity(180),
                        replenish_quantity: Quantity(20),
                    }),
            ),
        }),
    });
    println!("{}", outcome);

    // Submit an iceberg marketable sell order
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(101),
                QuantityPolicy::Iceberg {
                    visible_quantity: Quantity(10),
                    hidden_quantity: Quantity(90),
                    replenish_quantity: Quantity(10),
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
            order_id: target_order_id,
            order_kind: OrderKind::Limit,
        }),
    });
    println!("{}", outcome);

    // Submit iceberg buy orders
    for _ in 0..10 {
        let outcome = book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(10),
                        hidden_quantity: Quantity(90),
                        replenish_quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        println!("{}", outcome);
    }

    // Submit iceberg sell orders
    for _ in 0..10 {
        let outcome = book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(110),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(10),
                        hidden_quantity: Quantity(90),
                        replenish_quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        println!("{}", outcome);
    }

    // Submit iceberg marketable buy orders
    for _ in 0..5 {
        let outcome = book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(110),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(20),
                        hidden_quantity: Quantity(180),
                        replenish_quantity: Quantity(20),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
        println!("{}", outcome);
    }

    // Submit iceberg marketable sell orders
    for _ in 0..5 {
        let outcome = book.execute(&Command {
            meta: CommandMeta {
                sequence_number: helpers::sequence_number(),
                timestamp: helpers::now(),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(100),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(20),
                        hidden_quantity: Quantity(180),
                        replenish_quantity: Quantity(20),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        println!("{}", outcome);
    }
}
