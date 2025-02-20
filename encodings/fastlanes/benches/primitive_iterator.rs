fn main() {
    divan::main();
}

const BENCH_ARGS: &[usize] = &[
    2 << 10,
    2 << 12,
    2 << 14,
    2 << 16,
    2 << 18,
    2 << 20,
    2 << 22,
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
            PrimitiveArray::from_iter((0..args).map(|_| rng.gen_range::<u32, _>(0..1000))),
            args / (2 << 6),
            args - (args / 2),
        )
        .unwrap()
        .into_primitive()
        .unwrap();

        bencher
            .with_inputs(|| primitive_array.clone())
            .bench_local_values(|arr| {
                let mut iter = arr.iter_blocks::<u32, BLOCK_SIZE>();
                let mut sum = 0;
                loop {
                    let n = iter.next();
                    match n {
                        None => break,
                        Some(Ok(block)) => {
                            for i in block {
                                sum += i;
                            }
                        }
                        Some(Err(slice)) => {
                            for i in slice {
                                sum += i;
                            }
                        }
                    }
                }
                let _ = black_box(sum);
            })
    }

    #[divan::bench(args=BENCH_ARGS)]
    fn primitive_sum(bencher: Bencher, args: usize) {
        let mut rng = StdRng::seed_from_u64(0);
        let primitive_array = slice(
            PrimitiveArray::from_iter((0..args).map(|_| rng.gen_range::<u32, _>(0..1000))),
            args / (2 << 6),
            args - (args / 2),
        )
        .unwrap()
        .into_primitive()
        .unwrap();

        // println!("{}", primitive_array.len());

        bencher
            .with_inputs(|| primitive_array.clone())
            .bench_local_values(|arr| {
                let mut sum = 0;
                for i in arr.as_slice::<u32>() {
                    sum += i;
                }
                let _ = black_box(sum);
                // println!("s {s}")
            })
    }
}

mod bitpacked {
    use std::hint::black_box;

    use divan::Bencher;
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
                    (0..args).map(|_| rng.gen_range::<u32, _>(0..1000)),
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
                let mut iter = arr.iter_blocks::<_, u32, BLOCK_SIZE>();
                let mut sum = 0;
                loop {
                    let n = iter.next();
                    match n {
                        None => break,
                        Some(Ok(block)) => {
                            for i in block {
                                sum += i;
                            }
                        }
                        Some(Err(slice)) => {
                            for i in slice {
                                sum += i;
                            }
                        }
                    }
                }
                let _ = black_box(sum);
                // println!("{}", s);
            })
    }

    #[divan::bench(consts=CONST, args=BENCH_ARGS)]
    fn primitive_box_bp_iter_sum<const BLOCK_SIZE: usize>(bencher: Bencher, args: usize) {
        let mut rng = StdRng::seed_from_u64(0);
        let array = BitPackedArray::maybe_from(
            slice(
                bitpack_to_best_bit_width(PrimitiveArray::from_iter(
                    (0..args).map(|_| rng.gen_range::<u32, _>(0..1000)),
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
                let mut iter = arr.dyn_iter_blocks::<_, u32, BLOCK_SIZE>();
                let mut sum = 0;
                loop {
                    let n = iter.next();
                    match n {
                        None => break,
                        Some(Ok(block)) => {
                            for i in block {
                                sum += i;
                            }
                        }
                        Some(Err(slice)) => {
                            for i in slice {
                                sum += i;
                            }
                        }
                    }
                }
                let _ = black_box(sum);
            })
    }

    #[divan::bench(args=BENCH_ARGS)]
    fn primitive_bp_sum(bencher: Bencher, args: usize) {
        let mut rng = StdRng::seed_from_u64(0);
        let array = BitPackedArray::maybe_from(
            slice(
                bitpack_to_best_bit_width(PrimitiveArray::from_iter(
                    (0..args).map(|_| rng.gen_range::<u32, _>(0..1000)),
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
                let arr = arr.into_primitive().unwrap();
                let mut sum = 0;
                for i in arr.as_slice::<u32>() {
                    sum += i;
                }
                let _ = black_box(sum);
            })
    }
}
