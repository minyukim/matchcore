//! **Matchcore** is a high-performance order book and price-time matching engine implemented as a **single-threaded, deterministic, in-memory state machine**.
//!
//! It is designed for building **low-latency trading systems, exchange simulators, and market-microstructure research tools**.
//!
//! The architecture follows principles popularized by the [LMAX Architecture](https://martinfowler.com/articles/lmax.html), prioritizing deterministic execution, minimal synchronization, and predictable performance.
//!
//! # Features
//!
//! - Price-time priority matching engine
//! - Deterministic state machine execution
//! - Single-threaded design for minimal latency
//! - Efficient in-memory order book
//! - Support for advanced order types and flags (e.g., iceberg, pegged, time-in-force)
//! - Designed for integration with event-driven trading systems
//! - Clear command → outcome model for reproducible execution
//!
//! # What’s New in v0.3
//!
//! This release focuses on performance optimizations.
//!
//! - **In-place level updates** [#132](https://github.com/minyukim/matchcore/pull/132)
//!
//!   Orders are now updated in place instead of being removed and reinserted into the HashMap.
//!   This reduces overhead and improves price amendment performance by **~40%**.
//!
//! - **Switch to FxHashMap** [#135](https://github.com/minyukim/matchcore/pull/135)
//!
//!   Replaces the standard HashMap (SipHash) with [FxHashMap](https://docs.rs/rustc-hash/latest/rustc_hash/type.FxHashMap.html), a fast non-cryptographic hasher optimized for integer-heavy workloads.
//!   This significantly improves overall performance, especially:
//!
//!   - Cancellation throughput: **~40-52% faster**
//!   - Large-volume matching: **~35-60% faster**
//!
//! # Architecture
//!
//! The design is heavily inspired by the **LMAX architecture**, a model widely used in low-latency trading systems.
//!
//! Core principles include:
//!
//! - **Single-threaded state machine**
//! - **Event-driven command processing**
//! - **Deterministic execution**
//! - **In-memory data structures**
//!
//! These design choices eliminate synchronization overhead while guaranteeing reproducible behavior.
//!
//! ## Single-threaded
//!
//! For an order book of a **single instrument**, events must be processed **strictly sequentially**.
//!
//! Each event mutates the state of the book and the result of one event directly affects the next. Parallelizing matching for the same instrument therefore provides no performance benefit while introducing locking, contention, and complexity.
//!
//! Running the matching engine on a **single thread** provides several advantages:
//!
//! - No locks, contention, or synchronization overhead
//! - Predictable latency
//! - Simpler correctness guarantees
//!
//! This does **not** mean the entire application must be single-threaded.
//!
//! A typical architecture may look like:
//!
//! ```text
//! Command Reader/Decoder → Ring Buffer → Matchcore Engine → Ring Buffer → Execution Outcome Encoder/Writer
//! ```
//!
//! Systems can scale horizontally by **sharding instruments across multiple engine threads**.
//!
//! For example:
//!
//! ```text
//! Thread 1 → BTC-USD order book
//! Thread 2 → ETH-USD order book
//! Thread 3 → SOL-USD order book
//! ```
//!
//! ## Deterministic
//!
//! Matchcore operates as a **pure deterministic state machine**.
//!
//! Given:
//!
//! - The same initial state
//! - The same sequence of commands
//!
//! the engine will always produce **exactly the same results**.
//!
//! This property enables:
//!
//! - Deterministic replay
//! - Offline backtesting
//! - Simulation environments
//! - Auditability
//! - Event-sourced architectures
//!
//! Deterministic execution is particularly valuable for trading systems where correctness and reproducibility are critical.
//!
//! ## In-memory
//!
//! All state is maintained **entirely in memory**.
//!
//! The order book, price levels, and internal queues are optimized for fast access and minimal allocations.
//!
//! This design provides:
//!
//! - Extremely low latency
//! - Predictable performance
//! - Efficient memory access patterns
//!
//! Persistence and replication are expected to be handled **outside the engine**, typically through event logs and snapshots.
//!
//! # Core Concepts
//!
//! Matchcore processes **commands** and produces **outcomes**.
//!
//! ```text
//! Command → Matchcore Engine → Outcome
//! ```
//!
//! Commands represent user intent:
//!
//! - Submit order
//! - Amend order
//! - Cancel order
//!
//! Outcomes describe the result of execution:
//!
//! - Applied successfully
//! - Rejected because the command is invalid or cannot be executed in the current state of the order book
//!
//! Successfully applied commands may also produce:
//!
//! - Trades
//! - Order state changes
//! - Triggered orders
//!
//! ## Example
//!
//! ```rust
//! use matchcore::*;
//!
//! let mut book = OrderBook::new("ETH/USD");
//!
//! let outcome = book.execute(&Command {
//!     meta: CommandMeta {
//!         sequence_number: SequenceNumber(0),
//!         timestamp: Timestamp(1000),
//!     },
//!     kind: CommandKind::Submit(SubmitCmd {
//!         order: NewOrder::Limit(LimitOrder::new(
//!             Price(100),
//!             QuantityPolicy::Standard {
//!                 quantity: Quantity(10),
//!             },
//!             OrderFlags::new(Side::Buy, false /* post_only */, TimeInForce::Gtc),
//!         )),
//!     }),
//! });
//!
//! println!("{}", outcome);
//! ```
//!
//! More examples can be found in the [examples](examples) directory.
//!
//! # Supported Order Features
//!
//! Matchcore supports the following order types and execution options.
//!
//! ## Types
//!
//! - **Market Order**: executes immediately against the best available liquidity; optionally supports market-to-limit behavior if not fully filled
//! - **Limit Order**: executes at the specified price or better
//! - **Pegged Order**: dynamically reprices based on a reference price (e.g., best bid/ask)
//!
//! ## Flags
//!
//! - **Post-Only**: ensures the order adds liquidity only
//! - **Time-in-Force**: defines order lifetime (e.g., GTC, IOC, FOK, GTD)
//!
//! ## Quantity Policies
//!
//! - **Standard**: fully visible quantity
//! - **Iceberg**: partially visible quantity with hidden reserve that replenishes
//!
//! ## Peg References
//!
//! - **Primary**: pegs to the same-side best price (e.g., best bid for buy)
//! - **Market**: pegs to the opposite-side best price (e.g., best ask for buy)
//! - **Mid-Price**: pegs to the midpoint between best bid and best ask
//!
//! # Performance
//!
//! Benchmarks are run with [Criterion](https://bheisler.github.io/criterion.rs/book/).
//!
//! Matchcore is designed for low-latency, single-threaded, deterministic execution.
//!
//! Representative benchmark results measured on an Apple M4 using Rust stable are shown below.
//!
//! To run the benchmarks in your environment, run `make bench`.
//!
#![doc = include_str!("../docs/benchmarks.md")]
//!
//! ## Notes
//!
//! - Benchmark results depend on CPU, compiler version, benchmark configuration, and system load.
//! - These figures illustrate the general performance profile of the engine rather than serve as universal guarantees.
//! - Full Criterion output includes confidence intervals and regression comparisons.
//!
//! # Next Steps
//!
//! ## Additional Order Features
//!
//! - Stop orders
//! - Last-trade peg reference
//!
//! ## Potential Performance Improvements
//!
//! Currently, the order book stores price levels using `BTreeMap<Price, LevelId>` and `Slab<PriceLevel>`. This design provides:
//!
//! - **O(log N)** best-price lookup
//! - **O(log N)** submit operations to locate the corresponding price level
//! - **O(1)** amend operations (except when amending the order to a different price level)
//! - **O(1)** cancel operations (except when cancelling the order removes the price level entirely)
//!
//! where **N** is the number of price levels.
//!
//! An alternative design is to store prices in `Vec<(Price, LevelId)>`, sorted by price from **worst → best**, which provides:
//!
//! - **O(1)** best-price lookup
//! - **O(N)** insertion / deletion when creating or removing price levels
//!
//! However, in real-world trading scenarios, most activity occurs **near the best price**, meaning the effective search distance is often small. This can make a linear scan competitive with tree-based structures for typical workloads.

mod command;
mod orderbook;
mod orders;
mod outcome;
mod types;
mod utils;

pub use command::*;
pub use orderbook::*;
pub use orders::*;
pub use outcome::*;
pub use types::*;
