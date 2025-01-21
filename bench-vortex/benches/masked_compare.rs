//! Compare latency of three approaches to evaluating a masked comparison.
//!
//! Given an array `a` and a mask `m` of values of interest, produce an array containing `a_i < 0`
//! for each `i` where `m_i` is true.
//!
//! ```text
//! a = [-1, 0, 1, 2]
//! m = [true, false, true, false]
//! x = masked_lt0(a, m)
//! assert x == [true, false]
//! ```

use divan::Bencher;
use rand::rngs::StdRng;
use rand::{Rng as _, SeedableRng as _};
use vortex::array::BooleanBuffer;
use vortex::compute::FilterMask;

fn main() {
    divan::main();
}

#[divan::bench(args = [0.001, 0.01, 0.1, 0.5, 0.8])]
fn early_filter(bencher: Bencher, fraction_kept: f64) {
    let mut rng = StdRng::seed_from_u64(0);
    let data = (0..10000)
        .map(|_| rng.gen_range(0..100))
        .collect::<Vec<_>>();
    let mask = (0..10000)
        .map(|_| rng.gen_bool(fraction_kept))
        .collect::<BooleanBuffer>();
    let mask = FilterMask::from_buffer(mask);
    bencher.bench_local(move || {
        let filtered = mask
            .indices()
            .iter()
            .map(|kept_index| data[*kept_index])
            .collect::<Vec<_>>();
        filtered.iter().map(|value| *value < 0).collect::<Vec<_>>()
    });
}

#[divan::bench(args = [0.001, 0.01, 0.1, 0.5, 0.8])]
fn fused(bencher: Bencher, fraction_kept: f64) {
    let mut rng = StdRng::seed_from_u64(0);
    let data = (0..10000)
        .map(|_| rng.gen_range(0..100))
        .collect::<Vec<_>>();
    let mask = (0..10000)
        .map(|_| rng.gen_bool(fraction_kept))
        .collect::<BooleanBuffer>();
    let mask = FilterMask::from_buffer(mask);
    bencher.bench_local(move || {
        mask.indices()
            .iter()
            .map(|kept_index| data[*kept_index] < 0)
            .collect::<Vec<_>>()
    });
}

#[divan::bench(args = [0.001, 0.01, 0.1, 0.5, 0.8])]
fn late_filter(bencher: Bencher, fraction_kept: f64) {
    let mut rng = StdRng::seed_from_u64(0);
    let data = (0..10000)
        .map(|_| rng.gen_range(0..100))
        .collect::<Vec<_>>();
    let mask = (0..10000)
        .map(|_| rng.gen_bool(fraction_kept))
        .collect::<BooleanBuffer>();
    let mask = FilterMask::from_buffer(mask);
    bencher.bench_local(move || {
        let compared = data.iter().map(|x| *x < 0).collect::<Vec<_>>();
        mask.indices()
            .iter()
            .map(|kept_index| compared[*kept_index])
            .collect::<Vec<_>>()
    });
}

#[divan::bench()]
fn clone_and_compare_i32(bencher: Bencher) {
    let mut rng = StdRng::seed_from_u64(0);
    let data = (0..10000)
        .map(|_| rng.gen_range(0..100))
        .collect::<Vec<_>>();
    // let mask = (0..10000)
    //     .map(|_| rng.gen_bool(fraction_kept))
    //     .collect::<BooleanBuffer>();
    // let mask = FilterMask::from_buffer(mask);
    bencher.bench_local(move || data.clone());
}

#[divan::bench()]
fn clone_and_compare_bool(bencher: Bencher) {
    let mut rng = StdRng::seed_from_u64(0);
    let data = (0..10000).map(|_| rng.gen_bool(0.5)).collect::<Vec<_>>();
    // let mask = (0..10000)
    //     .map(|_| rng.gen_bool(fraction_kept))
    //     .collect::<BooleanBuffer>();
    // let mask = FilterMask::from_buffer(mask);
    bencher.bench_local(move || data.clone());
}

#[divan::bench(args = [0.001, 0.01, 0.1, 0.5, 0.8])]
fn filter_only(bencher: Bencher, fraction_kept: f64) {
    let mut rng = StdRng::seed_from_u64(0);
    let data = (0..10000)
        .map(|_| rng.gen_range(0..100))
        .collect::<Vec<_>>();
    let mask = (0..10000)
        .map(|_| rng.gen_bool(fraction_kept))
        .collect::<BooleanBuffer>();
    let mask = FilterMask::from_buffer(mask);
    bencher.bench_local(move || {
        mask.indices()
            .iter()
            .map(|kept_index| data[*kept_index])
            .collect::<Vec<_>>()
    });
}

#[divan::bench()]
fn compare_only(bencher: Bencher) {
    let mut rng = StdRng::seed_from_u64(0);
    let data = (0..10000)
        .map(|_| rng.gen_range(0..100))
        .collect::<Vec<_>>();
    bencher.bench_local(move || data.iter().map(|value| *value < 0).collect::<Vec<_>>());
}
