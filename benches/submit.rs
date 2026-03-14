use criterion::Criterion;
use std::hint::black_box;

use matchcore::*;

/// Benchmarks for submitting orders to an order book
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
                let result = book.execute(black_box(command));
                black_box(result);
            }
            black_box(book);
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
                let result = book.execute(black_box(command));
                black_box(result);
            }
            black_box(book);
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
                let result = book.execute(black_box(command));
                black_box(result);
            }
            black_box(book);
        })
    });

    group.finish();
}
