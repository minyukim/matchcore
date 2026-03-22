//! Benchmarks for matching orders in an order book
//!
//! Run: cargo bench --bench benches -- matching/

use criterion::{BatchSize, Criterion};
use std::hint::black_box;

use matchcore::*;

pub fn benches_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("matching");

    let n_orders = 1_000;

    for match_volume in [1, 10, 100, 1000, 10000] {
        let command = Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(n_orders),
                timestamp: Timestamp(n_orders),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(match_volume), Side::Buy, false)),
            }),
        };
        group.bench_function(
            format!("single_level_standard_book_match_volume_{}", match_volume),
            |b| {
                b.iter_batched(
                    || build_single_level_standard_book(n_orders),
                    |mut book| {
                        let outcome = book.execute(black_box(&command));
                        black_box(outcome);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    for match_volume in [1, 10, 100, 1000, 10000] {
        let command = Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(n_orders),
                timestamp: Timestamp(n_orders),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(match_volume), Side::Buy, false)),
            }),
        };
        group.bench_function(
            format!("multi_level_standard_book_match_volume_{}", match_volume),
            |b| {
                b.iter_batched(
                    || build_multi_level_standard_book(n_orders),
                    |mut book| {
                        let outcome = book.execute(black_box(&command));
                        black_box(outcome);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    for match_volume in [1, 10, 100, 1000, 10000] {
        let command = Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(n_orders),
                timestamp: Timestamp(n_orders),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(match_volume), Side::Buy, false)),
            }),
        };
        group.bench_function(
            format!("single_level_iceberg_book_match_volume_{}", match_volume),
            |b| {
                b.iter_batched(
                    || build_single_level_iceberg_book(n_orders),
                    |mut book| {
                        let outcome = book.execute(black_box(&command));
                        black_box(outcome);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    for match_volume in [1, 10, 100, 1000, 10000] {
        let command = Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(n_orders),
                timestamp: Timestamp(n_orders),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Market(MarketOrder::new(Quantity(match_volume), Side::Buy, false)),
            }),
        };
        group.bench_function(
            format!("multi_level_iceberg_book_match_volume_{}", match_volume),
            |b| {
                b.iter_batched(
                    || build_multi_level_iceberg_book(n_orders),
                    |mut book| {
                        let outcome = book.execute(black_box(&command));
                        black_box(outcome);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Helper function to build a single-level standard order book with `n_orders` orders
fn build_single_level_standard_book(n_orders: u64) -> OrderBook {
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
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
    }

    book
}

/// Helper function to build a multi-level standard order book with `n_orders` orders
fn build_multi_level_standard_book(n_orders: u64) -> OrderBook {
    let mut book = OrderBook::new("TEST");

    for i in 0..n_orders {
        let price = 100 + (i % 10);
        book.execute(&Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(i),
                timestamp: Timestamp(i),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(price),
                    QuantityPolicy::Standard {
                        quantity: Quantity(10),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
    }

    book
}

/// Helper function to build a single-level iceberg order book with `n_orders` orders
fn build_single_level_iceberg_book(n_orders: u64) -> OrderBook {
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
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(2),
                        hidden_quantity: Quantity(8),
                        replenish_quantity: Quantity(2),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
    }

    book
}

/// Helper function to build a multi-level iceberg order book with `n_orders` orders
fn build_multi_level_iceberg_book(n_orders: u64) -> OrderBook {
    let mut book = OrderBook::new("TEST");

    for i in 0..n_orders {
        let price = 100 + (i % 10);
        book.execute(&Command {
            meta: CommandMeta {
                sequence_number: SequenceNumber(i),
                timestamp: Timestamp(i),
            },
            kind: CommandKind::Submit(SubmitCmd {
                order: NewOrder::Limit(LimitOrder::new(
                    Price(price),
                    QuantityPolicy::Iceberg {
                        visible_quantity: Quantity(2),
                        hidden_quantity: Quantity(8),
                        replenish_quantity: Quantity(2),
                    },
                    OrderFlags::new(Side::Sell, false, TimeInForce::Gtc),
                )),
            }),
        });
    }

    book
}
