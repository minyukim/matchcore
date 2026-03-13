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

    group.finish();
}
