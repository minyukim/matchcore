//! Benchmarks for the matchcore library
//!
//! Run: cargo bench --bench benches

use criterion::{criterion_group, criterion_main};

mod amend;
mod cancel;
mod matching;
mod mixed;
mod submit;

criterion_group!(
    benches,
    submit::benches_submit,
    amend::benches_amend,
    cancel::benches_cancel,
    matching::benches_matching,
    mixed::benches_mixed,
);

criterion_main!(benches);
