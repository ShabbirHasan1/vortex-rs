//             if view.is_inlined() {
//                 view
//             } else {
//                 // Referencing views must have their buffer_index adjusted with new offsets
//                 let view_ref = view.as_view();
//                 BinaryView::new_view(
//                     view.len(),
//                     *view_ref.prefix(),
//                     buffers_offset + view_ref.buffer_index(),
//                     view_ref.offset(),
//                 )
//             }

use divan::Bencher;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use vortex_array::array::{map_views, map_views_crazy, BinaryView};

fn main() {
    divan::main();
}

const LEN: usize = 1000000;

#[divan::bench(args=[0.01, 0.1, 0.5, 0.9, 0.99])]
fn bench_view_inlined(bencher: Bencher, inline_prod: f64) {
    let mut rng = StdRng::seed_from_u64(23324);

    let views = (0..LEN)
        .map(|_| {
            if rng.gen_bool(inline_prod) {
                BinaryView::new_inlined(&[])
            } else {
                BinaryView::new_view(
                    rng.gen_range(0..1000),
                    [0u8; 4],
                    rng.gen_range(0..10),
                    rng.gen_range(0..1000),
                )
            }
        })
        .collect::<Vec<_>>();

    let buffers_offset = 10;

    bencher
        .with_inputs(|| views.clone())
        .bench_local_values(|views| {
            map_views(views, buffers_offset)
            // println!("res {:?}", res);
        });
}

#[divan::bench(args=[0.01, 0.1, 0.5, 0.9, 0.99])]
fn bless_bench_view_inlined(bencher: Bencher, inline_prod: f64) {
    let mut rng = StdRng::seed_from_u64(23324);

    let views = (0..LEN)
        .map(|_| {
            if rng.gen_bool(inline_prod) {
                BinaryView::new_inlined(&[])
            } else {
                BinaryView::new_view(
                    rng.gen_range(0..1000),
                    [0u8; 4],
                    rng.gen_range(0..10),
                    rng.gen_range(0..1000),
                )
            }
        })
        .collect::<Vec<_>>();

    let buffers_offset = 10;

    bencher
        .with_inputs(|| views.clone())
        .bench_local_values(|views| {
            map_views_crazy(views, buffers_offset)
            // println!("res {:?}", res);
        });
    // map_views_crazy(views, buffers_offset)
}
