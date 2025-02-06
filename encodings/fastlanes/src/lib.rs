#![allow(incomplete_features)]
#![allow(clippy::cast_possible_truncation)]
#![feature(generic_const_exprs)]
#![feature(vec_into_raw_parts)]
#![feature(iter_array_chunks)]

pub use bitpacking::*;
pub use delta::*;
pub use r#for::*;

mod bitpacking;
mod delta;
mod r#for;

#[cfg(test)]
mod tests {
    use rand::prelude::StdRng;
    use rand::{Rng, SeedableRng};
    use vortex_array::array::ChunkedArray;
    use vortex_array::builders::{ArrayBuilder, PrimitiveBuilder};
    use vortex_array::{Array, IntoArray, IntoArrayVariant, IntoCanonical};
    use vortex_buffer::BufferMut;
    use vortex_dtype::NativePType;

    use crate::bitpack_to_best_bit_width;

    fn make_array<T: NativePType>(len: usize) -> Array {
        let mut rng = StdRng::seed_from_u64(0);
        let values = (0..len)
            .map(|_| T::from(rng.gen_range(0..100)).unwrap())
            .collect::<BufferMut<T>>()
            .into_array()
            .into_primitive()
            .unwrap();

        bitpack_to_best_bit_width(values).unwrap().into_array()
    }

    #[test]
    fn test_canonical_into() {
        let len = 10000;
        let chunk = 100;
        let chunks = (0..chunk)
            .map(|_| make_array::<u32>(len))
            .collect::<Vec<_>>();
        let arr = make_array::<u32>(1);
        let chunked = ChunkedArray::try_new(chunks, arr.dtype().clone())
            .unwrap()
            .into_array();

        let into_ca = chunked
            .clone()
            .into_canonical()
            .unwrap()
            .into_primitive()
            .unwrap();
        let mut primitive_builder =
            PrimitiveBuilder::<u32>::with_capacity(arr.dtype().nullability(), len * chunk);
        chunked
            .clone()
            .canonicalize_into(&mut primitive_builder)
            .unwrap();
        let ca_into = primitive_builder.finish().unwrap();

        assert_eq!(
            into_ca.as_slice::<u32>(),
            ca_into.clone().into_primitive().unwrap().as_slice::<u32>()
        );

        // println!("into_ca: {:?}", into_ca.as_slice::<u32>());
        // println!(
        //     "ca_into: {:?}",
        //     ca_into.into_primitive().unwrap().as_slice::<u32>()
        // );

        let mut primitive_builder =
            PrimitiveBuilder::<u32>::with_capacity(arr.dtype().nullability(), 10 * 100);
        primitive_builder
            .extend_from_array(chunked.clone())
            .unwrap();
        let ca_into = primitive_builder.finish().unwrap();

        assert_eq!(
            into_ca.as_slice::<u32>(),
            ca_into.into_primitive().unwrap().as_slice::<u32>()
        );
    }
}
