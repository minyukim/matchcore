[![Crates.io][crates-badge]][crates-url]
[![Build Status][ci-badge]][ci-url]
[![Docs][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/matchcore
[crates-url]: https://crates.io/crates/matchcore
[ci-badge]: https://img.shields.io/github/actions/workflow/status/minyukim/matchcore/rust-ci.yml
[ci-url]: https://github.com/minyukim/matchcore/actions/workflows/rust-ci.yml
[docs-badge]: https://docs.rs/matchcore/badge.svg
[docs-url]: https://docs.rs/matchcore

# Matchcore

<!-- cargo-reedme: start -->

<!-- cargo-reedme: info-start

    Do not edit this region by hand
    ===============================

    This region was generated from Rust documentation comments by `cargo-reedme` using this command:

        cargo +nightly reedme

    for more info: https://github.com/nik-rev/cargo-reedme

cargo-reedme: info-end -->

**Matchcore** is a high-performance order book and price-time matching engine implemented as a **single-threaded, deterministic, in-memory state machine**.

It is designed for building **low-latency trading systems, exchange simulators, and market-microstructure research tools**.

The architecture follows principles popularized by the [LMAX Architecture](https://martinfowler.com/articles/lmax.html), prioritizing deterministic execution, minimal synchronization, and predictable performance.

## Features

- Price-time priority matching engine
- Deterministic state machine execution
- Single-threaded design for minimal latency
- Efficient in-memory order book
- Support for advanced order types and flags (e.g., iceberg, pegged, time-in-force)
- Designed for integration with event-driven trading systems
- Clear command → outcome model for reproducible execution

## Architecture

The design is heavily inspired by the **LMAX architecture**, a model widely used in low-latency trading systems.

Core principles include:

- **Single-threaded state machine**
- **Event-driven command processing**
- **Deterministic execution**
- **In-memory data structures**

These design choices eliminate synchronization overhead while guaranteeing reproducible behavior.

### Single-threaded

For an order book of a **single instrument**, events must be processed **strictly sequentially**.

Each event mutates the state of the book and the result of one event directly affects the next. Parallelizing matching for the same instrument therefore provides no performance benefit while introducing locking, contention, and complexity.

Running the matching engine on a **single thread** provides several advantages:

- No locks, contention, or synchronization overhead
- Predictable latency
- Simpler correctness guarantees

This does **not** mean the entire application must be single-threaded.

A typical architecture may look like:

```text
Command Reader/Decoder → Ring Buffer → Matchcore Engine → Ring Buffer → Execution Outcome Encoder/Writer
```

Systems can scale horizontally by **sharding instruments across multiple engine threads**.

For example:

```text
Thread 1 → BTC-USD order book
Thread 2 → ETH-USD order book
Thread 3 → SOL-USD order book
```

### Deterministic

Matchcore operates as a **pure deterministic state machine**.

Given:

- The same initial state
- The same sequence of commands

the engine will always produce **exactly the same results**.

This property enables:

- Deterministic replay
- Offline backtesting
- Simulation environments
- Auditability
- Event-sourced architectures

Deterministic execution is particularly valuable for trading systems where correctness and reproducibility are critical.

### In-memory

All state is maintained **entirely in memory**.

The order book, price levels, and internal queues are optimized for fast access and minimal allocations.

This design provides:

- Extremely low latency
- Predictable performance
- Efficient memory access patterns

Persistence and replication are expected to be handled **outside the engine**, typically through event logs and snapshots.

## Core Concepts

Matchcore processes **commands** and produces **outcomes**.

```text
Command → Matchcore Engine → Outcome
```

Commands represent user intent:

- Submit order
- Amend order
- Cancel order

Outcomes describe the result of execution:

- Applied successfully
- Rejected because the command is invalid or cannot be executed in the current state of the order book

Successfully applied commands may also produce:

- Trades
- Order state changes
- Triggered orders

### Example

```rust
use matchcore::*;

let mut book = OrderBook::new("ETH/USD");

let outcome = book.execute(&Command {
    meta: CommandMeta {
        sequence_number: SequenceNumber(0),
        timestamp: Timestamp(1000),
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
```

More examples can be found in the [examples](examples) directory.

## Supported Order Features

Matchcore supports the following order types and execution options.

### Types

- **Market Order**: executes immediately against the best available liquidity; optionally supports market-to-limit behavior if not fully filled
- **Limit Order**: executes at the specified price or better
- **Pegged Order**: dynamically reprices based on a reference price (e.g., best bid/ask)

### Flags

- **Post-Only**: ensures the order adds liquidity only
- **Time-in-Force**: defines order lifetime (e.g., GTC, IOC, FOK, GTD)

### Quantity Policies

- **Standard**: fully visible quantity
- **Iceberg**: partially visible quantity with hidden reserve that replenishes

### Peg References

- **Primary**: pegs to the same-side best price (e.g., best bid for buy)
- **Market**: pegs to the opposite-side best price (e.g., best ask for buy)
- **Mid-Price**: pegs to the midpoint between best bid and best ask

## Performance

Benchmarks are run with [Criterion](https://bheisler.github.io/criterion.rs/book/).

Matchcore is designed for low-latency, single-threaded, deterministic execution.

Representative benchmark results measured on an Apple M4 using Rust stable are shown below.

To run the benchmarks in your environment, run `make bench`.

### Submit

#### Single-order submit

| Benchmark | Time (median) |
| --- | ---: |
| Single standard order into a fresh book | ~112 ns |
| Single iceberg order into a fresh book | ~111 ns |
| Single post-only order into a fresh book | ~111 ns |
| Single good-till-date order into a fresh book | ~124 ns |
| Single pegged order into a fresh book | ~73 ns |

#### 10k orders submit

| Benchmark | Time (median) |
| --- | ---: |
| 10k standard orders into a fresh book | ~336.92 µs |
| 10k iceberg orders into a fresh book | ~333.22 µs |
| 10k post-only orders into a fresh book | ~334.43 µs |
| 10k good-till-date orders into a fresh book | ~348.38 µs |
| 10k pegged orders into a fresh book | ~275.76 µs |

### Amend

#### Single-order amend

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book quantity decrease | ~857 ns |
| Single order in multi-level book quantity decrease | ~696 ns |
| Single order in single-level book quantity increase | ~880 ns |
| Single order in multi-level book quantity increase | ~750 ns |
| Single order in single-level book price update | ~949 ns |
| Single order in multi-level book price update | ~777 ns |

#### 10k orders amend

| Benchmark | Time (median) |
| --- | ---: |
| 10k orders in single-level book quantity decrease | ~173.83 µs |
| 10k orders in multi-level book quantity decrease | ~165.64 µs |
| 10k orders in single-level book quantity increase | ~189.94 µs |
| 10k orders in multi-level book quantity increase | ~201.68 µs |
| 10k orders in single-level book price update | ~544.52 µs |
| 10k orders in multi-level book price update | ~520.13 µs |

### Cancel

| Benchmark | Time (median) |
| --- | ---: |
| Single order in single-level book cancel | ~867 ns |
| Single order in multi-level book cancel | ~709 ns |
| 10k orders in single-level book cancel | ~229.49 µs |
| 10k orders in multi-level book cancel | ~233.69 µs |

### Matching

#### Single-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~477 ns |
| 10 | ~484 ns |
| 100 | ~1.04 µs |
| 1000 | ~4.03 µs |
| 10000 | ~21.71 µs |

#### Multi-level standard book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~580 ns |
| 10 | ~596 ns |
| 100 | ~1.12 µs |
| 1000 | ~4.42 µs |
| 10000 | ~22.18 µs |

#### Single-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~476 ns |
| 10 | ~680 ns |
| 100 | ~2.27 µs |
| 1000 | ~8.78 µs |
| 10000 | ~69.19 µs |

#### Multi-level iceberg book

| Match volume | Time (median) |
| --- | ---: |
| 1 | ~580 ns |
| 10 | ~771 ns |
| 100 | ~2.29 µs |
| 1000 | ~8.46 µs |
| 10000 | ~68.38 µs |

### Mixed workload

| Benchmark | Time (median) |
| --- | ---: |
| Submit + amend + match + cancel | ~12.29 µs |

### Notes

- Benchmark results depend on CPU, compiler version, benchmark configuration, and system load.
- These figures illustrate the general performance profile of the engine rather than serve as universal guarantees.
- Full Criterion output includes confidence intervals and regression comparisons.

## Next Steps

### Additional Order Features

- Stop orders
- Last-trade peg reference

### Potential Performance Improvements

Currently, the order book stores price levels using `BTreeMap<Price, PriceLevel>`. This design provides:

- **O(log N)** best-price lookup
- **O(log N)** submit / amend / cancel operations to locate the corresponding price level

where **N** is the number of price levels.

Several alternative designs may improve performance.

#### 1. Slab-backed price levels

Use `Slab<PriceLevel>` and `BTreeMap<Price, LevelIdx>`, and each order holds its `LevelIdx`, allowing direct lookup of its price level. This would reduce the time complexity of the amend/cancel order operations to **O(1)**, except when cancelling the order removes the price level entirely.

#### 2. Sorted vector of price levels

Store price levels in `Vec<PriceLevel>`, sorted by price from **worst → best**.

Trade-offs:

- **O(1)** best-price lookup
- **O(N)** insertion / deletion when creating or removing price levels

However, in real-world trading scenarios, most activity occurs **near the best price**, meaning the effective search distance is often small. This can make a linear scan competitive with tree-based structures for typical workloads.

<!-- cargo-reedme: end -->

## Makefile

The project uses a Makefile to simplify the development process.

See the [Makefile](Makefile) for more details, or run `make` to see the available commands.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Contributions are welcome! If you would like to contribute, please follow these steps:

1. Fork the repository
2. Create a new branch for your changes
3. Make your changes
4. Run all the checks (`make check`)
5. Submit a pull request

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in matchcore by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
