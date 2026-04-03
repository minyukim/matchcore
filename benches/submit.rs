//! Benchmarks for submitting orders to an order book
//!
//! Run: cargo bench --bench benches -- submit/

use criterion::{BatchSize, Criterion};
use std::hint::black_box;

use matchcore::*;

pub fn benches_submit(c: &mut Criterion) {
    let mut group = c.benchmark_group("submit");

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(0),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            )),
        }),
    };
    group.bench_function("single_standard_order_fresh_book", |b| {
        b.iter(|| {
            let mut book: OrderBook = OrderBook::new("TEST");
            let outcome = book.execute(black_box(&command));
            black_box(outcome);
        })
    });

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(0),
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
    };
    group.bench_function("single_iceberg_order_fresh_book", |b| {
        b.iter(|| {
            let mut book: OrderBook = OrderBook::new("TEST");
            let outcome = book.execute(black_box(&command));
            black_box(outcome);
        })
    });

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(0),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, true, TimeInForce::Gtc),
            )),
        }),
    };
    group.bench_function("single_post_only_order_fresh_book", |b| {
        b.iter(|| {
            let mut book: OrderBook = OrderBook::new("TEST");
            let outcome = book.execute(black_box(&command));
            black_box(outcome);
        })
    });

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(0),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Limit(LimitOrder::new(
                Price(100),
                QuantityPolicy::Standard {
                    quantity: Quantity(100),
                },
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtd(Timestamp(100))),
            )),
        }),
    };
    group.bench_function("single_good_till_date_order_fresh_book", |b| {
        b.iter(|| {
            let mut book: OrderBook = OrderBook::new("TEST");
            let outcome = book.execute(black_box(&command));
            black_box(outcome);
        })
    });

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(0),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Pegged(PeggedOrder::new(
                PegReference::Primary,
                Quantity(100),
                OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
            )),
        }),
    };
    group.bench_function("single_pegged_order_fresh_book", |b| {
        b.iter(|| {
            let mut book: OrderBook = OrderBook::new("TEST");
            let outcome = book.execute(black_box(&command));
            black_box(outcome);
        })
    });

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(0),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::PriceConditional(PriceConditionalOrder::stop_market(
                Price(100),
                MarketOrder::new(Quantity(100), Side::Buy, false),
            )),
        }),
    };
    group.bench_function("single_price_conditional_order_fresh_book", |b| {
        b.iter(|| {
            let mut book: OrderBook = OrderBook::new("TEST");
            let outcome = book.execute(black_box(&command));
            black_box(outcome);
        })
    });

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(2),
            timestamp: Timestamp(0),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::PriceConditional(PriceConditionalOrder::stop_limit(
                Price(101),
                LimitOrder::new(
                    Price(101),
                    QuantityPolicy::Standard {
                        quantity: Quantity(100),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
            )),
        }),
    };
    group.bench_function("single_inactive_price_conditional_stop_limit_order", |b| {
        b.iter_batched(
            || populate_book_with_last_trade_price(Price(100)),
            |mut book| {
                let outcome = book.execute(black_box(&command));
                black_box(outcome);
            },
            BatchSize::SmallInput,
        )
    });

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(2),
            timestamp: Timestamp(0),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::PriceConditional(PriceConditionalOrder::stop_limit(
                Price(99),
                LimitOrder::new(
                    Price(99),
                    QuantityPolicy::Standard {
                        quantity: Quantity(100),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                ),
            )),
        }),
    };
    group.bench_function("single_active_price_conditional_stop_limit_order", |b| {
        b.iter_batched(
            || populate_book_with_last_trade_price(Price(100)),
            |mut book| {
                let outcome = book.execute(black_box(&command));
                black_box(outcome);
            },
            BatchSize::SmallInput,
        )
    });

    let commands: Vec<Command> = (0..10_000)
        .map(|i| {
            let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
            let price = Price(if side == Side::Buy {
                10_000 - 1 - (i % 10)
            } else {
                10_000 + 1 + (i % 10)
            });

            Command {
                meta: CommandMeta {
                    sequence_number: SequenceNumber(i),
                    timestamp: Timestamp(i),
                },
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        price,
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(side, false, TimeInForce::Gtc),
                    )),
                }),
            }
        })
        .collect();
    group.bench_function("10k_standard_orders_fresh_book", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("TEST");
            for command in &commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            black_box(book);
        })
    });

    let commands: Vec<Command> = (0..10_000)
        .map(|i| {
            let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
            let price = Price(if side == Side::Buy {
                10_000 - 1 - (i % 10)
            } else {
                10_000 + 1 + (i % 10)
            });

            Command {
                meta: CommandMeta {
                    sequence_number: SequenceNumber(i),
                    timestamp: Timestamp(i),
                },
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        price,
                        QuantityPolicy::Iceberg {
                            visible_quantity: Quantity(10),
                            hidden_quantity: Quantity(90),
                            replenish_quantity: Quantity(10),
                        },
                        OrderFlags::new(side, false, TimeInForce::Gtc),
                    )),
                }),
            }
        })
        .collect();
    group.bench_function("10k_iceberg_orders_fresh_book", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("TEST");
            for command in &commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            black_box(book);
        })
    });

    let commands: Vec<Command> = (0..10_000)
        .map(|i| {
            let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
            let price = Price(if side == Side::Buy {
                10_000 - 1 - (i % 10)
            } else {
                10_000 + 1 + (i % 10)
            });

            Command {
                meta: CommandMeta {
                    sequence_number: SequenceNumber(i),
                    timestamp: Timestamp(i),
                },
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        price,
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(side, true, TimeInForce::Gtc),
                    )),
                }),
            }
        })
        .collect();
    group.bench_function("10k_post_only_orders_fresh_book", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("TEST");
            for command in &commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            black_box(book);
        })
    });

    let commands: Vec<Command> = (0..10_000)
        .map(|i| {
            let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
            let price = Price(if side == Side::Buy {
                10_000 - 1 - (i % 10)
            } else {
                10_000 + 1 + (i % 10)
            });

            Command {
                meta: CommandMeta {
                    sequence_number: SequenceNumber(i),
                    timestamp: Timestamp(i),
                },
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Limit(LimitOrder::new(
                        price,
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(side, false, TimeInForce::Gtd(Timestamp(10_000))),
                    )),
                }),
            }
        })
        .collect();
    group.bench_function("10k_good_till_date_orders_fresh_book", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("TEST");
            for command in &commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            black_box(book);
        })
    });

    let commands: Vec<Command> = (0..10_000)
        .map(|i| {
            let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
            let reference = if i % 3 == 0 {
                PegReference::Primary
            } else if i % 3 == 1 {
                PegReference::Market
            } else {
                PegReference::MidPrice
            };

            Command {
                meta: CommandMeta {
                    sequence_number: SequenceNumber(i),
                    timestamp: Timestamp(i),
                },
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::Pegged(PeggedOrder::new(
                        reference,
                        Quantity(100),
                        OrderFlags::new(side, false, TimeInForce::Gtc),
                    )),
                }),
            }
        })
        .collect();
    group.bench_function("10k_pegged_orders_fresh_book", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("TEST");
            for command in &commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            black_box(book);
        })
    });

    let commands: Vec<Command> = (0..10_000)
        .map(|i| {
            let price = Price(100 + (i % 10));
            let order = match i % 4 {
                0 => PriceConditionalOrder::stop_market(
                    price,
                    MarketOrder::new(Quantity(100), Side::Buy, false),
                ),
                1 => PriceConditionalOrder::stop_market(
                    price,
                    MarketOrder::new(Quantity(100), Side::Sell, false),
                ),
                2 => PriceConditionalOrder::take_profit_market(
                    price,
                    MarketOrder::new(Quantity(100), Side::Buy, false),
                ),
                3 => PriceConditionalOrder::take_profit_market(
                    price,
                    MarketOrder::new(Quantity(100), Side::Sell, false),
                ),
                _ => unreachable!(),
            };

            Command {
                meta: CommandMeta {
                    sequence_number: SequenceNumber(i),
                    timestamp: Timestamp(i),
                },
                kind: CommandKind::Submit(SubmitCmd {
                    order: NewOrder::PriceConditional(order),
                }),
            }
        })
        .collect();
    group.bench_function("10k_price_conditional_orders_fresh_book", |b| {
        b.iter(|| {
            let mut book = OrderBook::new("TEST");
            for command in &commands {
                let outcome = book.execute(black_box(command));
                black_box(outcome);
            }
            black_box(book);
        })
    });

    let commands: Vec<Command> = (0..10_000)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(2 + i),
                timestamp: Timestamp(2 + i),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::PriceConditional(PriceConditionalOrder::stop_limit(
                    Price(101 + (i % 10)),
                    LimitOrder::new(
                        Price(101 + (i % 10)),
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                )),
            }),
        })
        .collect();
    group.bench_function("10k_inactive_price_conditional_stop_limit_orders", |b| {
        b.iter_batched(
            || populate_book_with_last_trade_price(Price(100)),
            |mut book| {
                for command in &commands {
                    let outcome = book.execute(black_box(command));
                    black_box(outcome);
                }
            },
            BatchSize::SmallInput,
        )
    });

    let commands: Vec<Command> = (0..10_000)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(2 + i),
                timestamp: Timestamp(2 + i),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::PriceConditional(PriceConditionalOrder::stop_limit(
                    Price(99 - (i % 10)),
                    LimitOrder::new(
                        Price(99 - (i % 10)),
                        QuantityPolicy::Standard {
                            quantity: Quantity(100),
                        },
                        OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                    ),
                )),
            }),
        })
        .collect();
    group.bench_function("10k_active_price_conditional_stop_limit_orders", |b| {
        b.iter_batched(
            || populate_book_with_last_trade_price(Price(100)),
            |mut book| {
                for command in &commands {
                    let outcome = book.execute(black_box(command));
                    black_box(outcome);
                }
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn populate_book_with_last_trade_price(trade_price: Price) -> OrderBook {
    let mut book = OrderBook::new("TEST");

    let _ = book.execute(&Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(0),
            timestamp: Timestamp(0),
        },
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
        meta: CommandMeta {
            sequence_number: SequenceNumber(1),
            timestamp: Timestamp(0),
        },
        kind: CommandKind::Submit(SubmitCmd {
            order: NewOrder::Market(MarketOrder::new(Quantity(1), Side::Buy, false)),
        }),
    });

    book
}
