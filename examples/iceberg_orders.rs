//! Example: submit, amend, cancel, and match iceberg orders
//!
//! Flow:
//! 1. Submit an iceberg bid (visible + hidden + replenish size).
//! 2. Amend price, visible size, and hidden size.
//! 3. Cross with an aggressive iceberg sell at the new touch.
//! 4. Cancel the remaining bid.
//! 5. Populate both sides with iceberg ladders, then submit aggressive iceberg limits that trade through the book.
//!
//! Run: `cargo run --example iceberg_orders`

mod helpers;

use matchcore::*;

fn main() {
    let mut book = OrderBook::new("ETH/USD");

    println!("=== Submit a resting bid @ 100 (visible 10, hidden 90, replenish 10) ===");
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
    println!("{outcome}");

    let target_order_id = helpers::target_order_id(&outcome).unwrap();

    println!(
        "=== Amend the bid price 100 -> 101, visible 10 -> 20, hidden 90 -> 180, replenish 10 -> 20 ==="
    );
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
    println!("{outcome}");

    println!("=== Submit a marketable ask @ 101 (total 100) ===");
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
    println!("{outcome}");

    println!("=== Cancel the remaining bid ===");
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
    println!("{outcome}");

    println!("=== Submit bids @ 100 (visible 10, hidden 90, replenish 10) ===");
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
        println!("{outcome}");
    }

    println!("=== Submit asks @ 110 (visible 10, hidden 90, replenish 10) ===");
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
        println!("{outcome}");
    }

    println!("=== Submit marketable bids @ 110 (total 200) ===");
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
        println!("{outcome}");
    }

    println!("=== Submit marketable asks @ 100 (total 200) ===");
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
        println!("{outcome}");
    }
}
