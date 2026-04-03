//! Example: submit, amend, cancel, and match pegged orders (primary, market, mid-price)
//!
//! Flow:
//! 1. Submit a primary pegged buy, then amend it to a market peg with larger quantity.
//! 2. Add a resting sell so the market peg can trade; cancel any remaining peg.
//! 3. Exercise mid-price and primary pegs with resting limits, then show market peg behavior when the
//!    spread is wide (mid-price inactive) vs tight (mid-price active).
//!
//! Run: `cargo run --example pegged_orders`

mod helpers;

use matchcore::*;

fn main() {
    let mut book = OrderBook::new("ETH/USD");

    println!("=== Submit a primary pegged bid ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Pegged(PeggedOrder::new(
                PegReference::Primary,
                Quantity(100),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);

    let target_order_id = helpers::target_order_id(&outcome).unwrap();

    println!("=== Amend the bid to become a market pegged bid (waiting for new sell order) ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Amend(AmendCmd {
            order_id: target_order_id,
            patch: AmendPatch::Pegged(
                PeggedOrderPatch::new()
                    .with_peg_reference(PegReference::Market)
                    .with_quantity(Quantity(200)),
            ),
        }),
    });
    println!("{}", outcome);

    println!("=== Submit a standard ask @ 110 (triggers the market pegged bid) ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(110),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);

    println!("=== Cancel the remaining pegged bid ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Cancel(CancelCmd {
            order_id: target_order_id,
            order_kind: OrderKind::Pegged,
        }),
    });
    println!("{}", outcome);

    println!("=== Submit a mid price pegged bid ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Pegged(PeggedOrder::new(
                PegReference::MidPrice,
                Quantity(100),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);

    println!("=== Submit a primary pegged bid ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Pegged(PeggedOrder::new(
                PegReference::Primary,
                Quantity(100),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);

    println!("=== Submit a standard bid @ 100 ===");
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

    println!("=== Submit a standard ask @ 110 ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(110),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);

    println!("=== Submit a market pegged ask (spread > 1 -> mid price inactive) ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Pegged(PeggedOrder::new(
                PegReference::Market,
                Quantity(90),
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);

    println!("=== Submit a standard bid @ 109 ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(109),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);

    println!("=== Submit a market pegged ask (spread <= 1 -> mid price active) ===");
    let outcome = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: helpers::sequence_number(),
            timestamp: helpers::now(),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Pegged(PeggedOrder::new(
                PegReference::Market,
                Quantity(130),
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{}", outcome);
}
