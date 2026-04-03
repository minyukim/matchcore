//! Example: submit, amend, cancel, and match-activated price-conditional orders
//!
//! Flow:
//! 1. Seed a last-trade price with a simple limit-vs-market trade.
//! 2. Submit a stop-limit (price-conditional) order that rests until the market trades through its trigger.
//! 3. Amend the resting order (tighter target limit).
//! 4. Submit another price-conditional order and cancel it before it can fire.
//! 5. Stack the book so a market buy walks the last-trade price up through the trigger band; the engine
//!    drains eligible triggers and submits the activated limit, which then matches against resting liquidity.
//!
//! Run: `cargo run --example price_conditional_orders`

mod helpers;

use matchcore::*;

fn meta() -> CommandMeta {
    CommandMeta {
        sequence_number: helpers::sequence_number(),
        timestamp: helpers::now(),
    }
}

fn seed_last_trade_price(book: &mut OrderBook, trade_price: Price) {
    let _ = book.execute(&Command {
        meta: meta(),
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                trade_price,
                QuantityPolicy::Standard {
                    quantity: Quantity(1),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            )),
        }),
    });

    let _ = book.execute(&Command {
        meta: meta(),
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Market(MarketOrder::new(Quantity(1), Side::Buy, false)),
        }),
    });

    assert_eq!(
        book.last_trade_price(),
        Some(trade_price),
        "seed trade should set last_trade_price"
    );
}

fn main() {
    let mut book = OrderBook::new("ETH/USD");

    println!("=== Seed last trade at 100 ===\n");
    seed_last_trade_price(&mut book, Price(100));

    println!("=== Submit stop-limit buy: trigger >= 105, activated limit @ 112 ===");
    let outcome = book.execute(&Command {
        meta: meta(),
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::PriceConditional(PriceConditionalOrder::stop_limit(
                Price(105),
                LimitOrder::new(
                    Price(112),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
            )),
        }),
    });
    println!("{outcome}");
    let stop_limit_id = helpers::target_order_id(&outcome).expect("submit should return order id");

    println!("=== Amend: tighten activated limit 112 -> 111 (same trigger) ===");
    let outcome = book.execute(&Command {
        meta: meta(),
        kind: CommandKind::Amend(AmendCmd {
            order_id: stop_limit_id,
            patch: AmendPatch::PriceConditional(
                PriceConditionalOrderPatch::new().with_target_order(TriggerOrder::Limit(
                    LimitOrder::new(
                        Price(111),
                        QuantityPolicy::Standard {
                            quantity: Quantity(10),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                )),
            ),
        }),
    });
    println!("{outcome}");

    println!("=== Submit & cancel a separate price-conditional order (never triggers) ===");
    let outcome = book.execute(&Command {
        meta: meta(),
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::PriceConditional(PriceConditionalOrder::new(
                PriceCondition::new(Price(500), TriggerDirection::AtOrAbove),
                TriggerOrder::Market(MarketOrder::new(Quantity(1), Side::Buy, false)),
            )),
        }),
    });
    println!("{outcome}");
    let disposable_id = helpers::target_order_id(&outcome).expect("submit should return order id");

    let outcome = book.execute(&Command {
        meta: meta(),
        kind: CommandKind::Cancel(CancelCmd {
            order_id: disposable_id,
            order_kind: OrderKind::PriceConditional,
        }),
    });
    println!("{outcome}");

    println!(
        "=== Stack asks: walk 101..=105 (1 lot each), then liquidity at 111 for the activated limit ==="
    );
    for p in 101u64..=105 {
        let outcome = book.execute(&Command {
            meta: meta(),
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(p),
                    QuantityPolicy::Standard {
                        quantity: Quantity(1),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
        println!("{outcome}");
    }

    let outcome = book.execute(&Command {
        meta: meta(),
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(111),
                QuantityPolicy::Standard {
                    quantity: Quantity(10),
                },
                OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
            )),
        }),
    });
    println!("{outcome}");

    println!(
        "=== Market buy 5: last trade 100 -> 105, triggers stop-limit; activated buy @ 111 matches ==="
    );
    let outcome = book.execute(&Command {
        meta: meta(),
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Market(MarketOrder::new(Quantity(5), Side::Buy, false)),
        }),
    });
    println!("{outcome}");

    println!("last_trade_price = {:?}", book.last_trade_price());
    println!(
        "price-conditional entry for id {:?} cleared? {}",
        stop_limit_id,
        !book
            .price_conditional()
            .orders()
            .contains_key(&stop_limit_id)
    );
    println!(
        "activated limit fully filled (no resting limit with same id)? {}",
        !book.limit().orders().contains_key(&stop_limit_id)
    );
}
