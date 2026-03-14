use criterion::{BatchSize, Criterion};
use std::hint::black_box;

use matchcore::*;

/// Benchmarks for amending orders in an order book
pub fn benches_amend(c: &mut Criterion) {
    let mut group = c.benchmark_group("amend");

    let n_orders = 10_000;

    let command = Command {
        meta: CommandMeta {
            sequence_number: SequenceNumber(n_orders),
            timestamp: Timestamp(n_orders),
        },
        kind: CommandKind::Amend(AmendCmd {
            order_id: OrderId(0),
            patch: AmendPatch::Limit(LimitOrderPatch::new().with_quantity_policy(
                QuantityPolicy::Standard {
                    quantity: Quantity(50),
                },
            )),
        }),
    };
    group.bench_function("single_order_quantity_decrease", |b| {
        b.iter_batched(
            || build_book(n_orders),
            |mut book| {
                let outcome = book.execute(black_box(&command));
                black_box(outcome);
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

/// Helper function to build an order book with `n_orders` standard orders
fn build_book(n_orders: u64) -> OrderBook {
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
