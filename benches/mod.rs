use criterion::{criterion_group, criterion_main};

mod amend;
mod submit;

criterion_group!(benches, submit::benches_submit, amend::benches_amend);

criterion_main!(benches);
