//! Example: submit, amend, cancel, and match standard orders
//!
//! Flow:
//! 1. Submit a resting bid, then amend price and size.
//! 2. Cross it partially with an aggressive limit sell.
//! 3. Cancel what remains of the original bid.
//! 4. Build a two-sided ladder (bids stepping down from 100, asks stepping up from 110).
//! 5. Fire aggressive limit buys and sells that walk the book (marketable limits).
//!
//! Run: `cargo run --example standard_orders`

mod helpers;

use matchcore::*;

fn main() {
    let mut book = OrderBook::new("ETH/USD");

    println!("=== Submit a resting bid @ 100 ===");
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
    println!("{outcome}");
    let target_order_id = helpers::target_order_id(&outcome).unwrap();

    println!("=== Amend the bid price 100 -> 101 and size 10 -> 20 ===");
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
    println!("{outcome}");

    println!("=== Submit a marketable ask @ 101 ===");
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
        println!("{outcome}");
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
        println!("{outcome}");
    }

    println!("=== Submit marketable bids @ 120 ===");
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
        println!("{outcome}");
    }

    println!("=== Submit marketable asks @ 90 ===");
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
        println!("{outcome}");
    }
}
