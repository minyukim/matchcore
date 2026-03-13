use criterion::{criterion_group, criterion_main};

mod submit;

criterion_group!(benches, submit::benches_submit);

criterion_main!(benches);
