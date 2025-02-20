fn main() {
    divan::main();
}

const BENCH_ARGS: &[usize] = &[
    // 2 << 10,
    // 2 << 12,
    // 2 << 14,
    // 2 << 16,
    2 << 18,
    2 << 20,
    2 << 22,
    2 << 24,
    // 2 << 26,
];

const CONST: [usize; 1] = [
    2 << 9,
    // 2 << 10,
    // 2 << 11,
    // 2 << 12,
    // 2 << 13,
    // 2 << 14,
    // 2 << 15,
    // 2 << 16,
];

mod primitive {
    use std::hint::black_box;

    use divan::Bencher;
    use itertools::Itertools;
    use rand::prelude::StdRng;
    use rand::{Rng, SeedableRng};
    use vortex_array::accessor::ArrayValueIterator;
    use vortex_array::arrays::PrimitiveArray;
    use vortex_array::compute::slice;
    use vortex_array::IntoArrayVariant;

    use crate::{BENCH_ARGS, CONST};

    #[divan::bench(consts=CONST, args=BENCH_ARGS)]
    fn primitive_iterator_sum<const BLOCK_SIZE: usize>(bencher: Bencher, args: usize) {
        let mut rng = StdRng::seed_from_u64(0);
        let primitive_array = slice(
            PrimitiveArray::from_iter((0..args).map(|_| rng.gen_range::<i32, _>(0..1000))),
            args / (2 << 6),
            args - (args / 2),
        )
        .unwrap()
        .into_primitive()
        .unwrap();

        bencher
            .with_inputs(|| primitive_array.clone())
            .bench_local_values(|arr| {
                let mut iter = arr.iter_blocks::<i32, BLOCK_SIZE>();
                let mut out_block = [0i32; BLOCK_SIZE];
                loop {
                    let n = iter.next();
                    match n {
                        None => break,
                        Some(Ok(block)) => {
                            for i in 0..block.len() {
                                out_block[i] = block[i] + 1;
                            }
                        }
                        Some(Err(slice)) => {
                            for i in 0..slice.len() {
                                out_block[i] = slice[i] + 1;
                            }
                        }
                    }
                }
            })
    }

    #[divan::bench(args=BENCH_ARGS)]
    fn primitive_sum(bencher: Bencher, args: usize) {
        let mut rng = StdRng::seed_from_u64(0);
        let primitive_array = slice(
            PrimitiveArray::from_iter((0..args).map(|_| rng.gen_range::<i64, _>(0..1000))),
            args / (2 << 6),
            args - (args / 2),
        )
        .unwrap()
        .into_primitive()
        .unwrap();

        bencher
            .with_inputs(|| primitive_array.clone())
            .bench_local_values(|arr| {
                let values = arr
                    .as_slice::<i64>()
                    .into_iter()
                    .map(|x| (x + 1) as i32)
                    .collect_vec();
                black_box(values);
            })
    }
}
//
mod bitpacked {
    use std::hint::black_box;

    use divan::Bencher;
    use itertools::Itertools;
    use rand::prelude::StdRng;
    use rand::{Rng, SeedableRng};
    use vortex_array::accessor::ArrayValueIterator;
    use vortex_array::arrays::PrimitiveArray;
    use vortex_array::compute::slice;
    use vortex_array::IntoArrayVariant;
    use vortex_fastlanes::{bitpack_to_best_bit_width, BitPackedArray};

    use crate::{BENCH_ARGS, CONST};

    #[divan::bench(consts=CONST, args=BENCH_ARGS)]
    fn primitive_bp_iter_sum<const BLOCK_SIZE: usize>(bencher: Bencher, args: usize) {
        let mut rng = StdRng::seed_from_u64(0);
        let array = BitPackedArray::maybe_from(
            slice(
                bitpack_to_best_bit_width(PrimitiveArray::from_iter(
                    (0..args).map(|_| rng.gen_range::<i64, _>(0..1000)),
                ))
                .unwrap(),
                args / (2 << 6),
                args - (args / 2),
            )
            .unwrap(),
        )
        .unwrap();

        bencher
            .with_inputs(|| array.clone())
            .bench_local_values(|arr| {
                let mut iter = arr.iter_blocks::<i32, BLOCK_SIZE>();
                let mut out_block = [0i32; BLOCK_SIZE];
                loop {
                    let n = iter.next();
                    match n {
                        None => break,
                        Some(Ok(block)) => {
                            for i in 0..block.len() {
                                out_block[i] = block[i] + 1;
                            }
                        }
                        Some(Err(slice)) => {
                            for i in 0..slice.len() {
                                out_block[i] = slice[i] + 1;
                            }
                        }
                    }
                }
            })
    }

    #[divan::bench(consts=CONST, args=BENCH_ARGS)]
    fn primitive_box_bp_iter_sum<const BLOCK_SIZE: usize>(bencher: Bencher, args: usize) {
        let mut rng = StdRng::seed_from_u64(0);
        let array = BitPackedArray::maybe_from(
            slice(
                bitpack_to_best_bit_width(PrimitiveArray::from_iter(
                    (0..args).map(|_| rng.gen_range::<i64, _>(0..1000)),
                ))
                .unwrap(),
                args / (2 << 6),
                args - (args / 2),
            )
            .unwrap(),
        )
        .unwrap();

        bencher
            .with_inputs(|| array.clone())
            .bench_local_values(|arr| {
                let mut iter = arr.dyn_iter_blocks::<i32, BLOCK_SIZE>();
                let mut out_block = [0i32; BLOCK_SIZE];
                loop {
                    let n = iter.next();
                    match n {
                        None => break,
                        Some(Ok(block)) => {
                            for i in 0..block.len() {
                                out_block[i] = block[i] + 1;
                            }
                        }
                        Some(Err(slice)) => {
                            for i in 0..slice.len() {
                                out_block[i] = slice[i] + 1;
                            }
                        }
                    }
                }
            })
    }

    #[divan::bench(args=BENCH_ARGS)]
    fn primitive_bp_sum(bencher: Bencher, args: usize) {
        let mut rng = StdRng::seed_from_u64(0);
        let array = BitPackedArray::maybe_from(
            slice(
                bitpack_to_best_bit_width(PrimitiveArray::from_iter(
                    (0..args).map(|_| rng.gen_range::<i64, _>(0..1000)),
                ))
                .unwrap(),
                args / (2 << 6),
                args - (args / 2),
            )
            .unwrap(),
        )
        .unwrap();

        bencher
            .with_inputs(|| array.clone())
            .bench_local_values(|arr| {
                let values = arr
                    .into_primitive()
                    .unwrap()
                    .as_slice::<i64>()
                    .into_iter()
                    .map(|x| (x + 1) as i32)
                    .collect_vec();
                black_box(values)
            })
    }
}
