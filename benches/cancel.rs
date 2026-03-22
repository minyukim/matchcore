//! Benchmarks for canceling orders in an order book
//!
//! Run: cargo bench --bench benches -- cancel/

use criterion::{BatchSize, Criterion};
use std::hint::black_box;

use matchcore::*;

pub fn benches_cancel(c: &mut Criterion) {
    let mut group = c.benchmark_group("cancel");

    let n_orders = 10_000;

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(n_orders),
            timestamp: Timestamp(n_orders),
        },
        kind: CommandKind::Cancel(CancelCmd {
            order_id: OrderId(0),
            order_kind: OrderKind::Limit,
        }),
    };
    group.bench_function("single_order_in_single_level_book_cancel", |b| {
        b.iter_batched(
            || build_single_level_book(n_orders),
            |mut book| {
                let outcome = book.execute(black_box(&command));
                black_box(outcome);
            },
            BatchSize::SmallInput,
        )
    });

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(n_orders),
            timestamp: Timestamp(n_orders),
        },
        kind: CommandKind::Cancel(CancelCmd {
            order_id: OrderId(n_orders / 2),
            order_kind: OrderKind::Limit,
        }),
    };
    group.bench_function("single_order_in_multi_level_book_cancel", |b| {
        b.iter_batched(
            || build_multi_level_book(n_orders),
            |mut book| {
                let outcome = book.execute(black_box(&command));
                black_box(outcome);
            },
            BatchSize::SmallInput,
        )
    });

    let commands: Vec<Command> = (0..n_orders)
        .map(|i| Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(n_orders + i),
                timestamp: Timestamp(n_orders + i),
            },
            kind: CommandKind::Cancel(CancelCmd {
                order_id: OrderId(i),
                order_kind: OrderKind::Limit,
            }),
        })
        .collect();
    group.bench_function("10k_orders_in_single_level_book_cancel", |b| {
        b.iter_batched(
            || build_single_level_book(n_orders),
            |mut book| {
                for command in &commands {
                    let outcome = book.execute(black_box(command));
                    black_box(outcome);
                }
            },
            BatchSize::SmallInput,
        )
    });
    group.bench_function("10k_orders_in_multi_level_book_cancel", |b| {
        b.iter_batched(
            || build_multi_level_book(n_orders),
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

/// Helper function to build an order book with `n_orders` standard orders in a single level
fn build_single_level_book(n_orders: u64) -> OrderBook {
    let mut book = OrderBook::new("TEST");

    for i in 0..n_orders {
        book.execute(&Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(i),
                timestamp: Timestamp(i),
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
        });
    }

    book
}

/// Helper function to build an order book with `n_orders` standard orders in multiple levels
fn build_multi_level_book(n_orders: u64) -> OrderBook {
    let mut book = OrderBook::new("TEST");

    for i in 0..n_orders {
        book.execute(&Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(i),
                timestamp: Timestamp(i),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(100 + (i % 10)),
                    QuantityPolicy::Standard {
                        quantity: Quantity(100),
                    },
                    OrderFlags::new(Side::Buy, false, TimeInForce::Gtc),
                )),
            }),
        });
    }

    book
}
