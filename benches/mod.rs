use criterion::{criterion_group, criterion_main};

mod amend;
mod cancel;
mod submit;

criterion_group!(
    benches,
    submit::benches_submit,
    amend::benches_amend,
    cancel::benches_cancel
);

criterion_main!(benches);
