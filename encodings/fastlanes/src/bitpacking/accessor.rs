use std::marker::PhantomData;

use fastlanes::BitPacking;
use num_traits::Zero;
use vortex_array::accessor::ArrayValueIterator;
use vortex_dtype::NativePType;

use crate::{BitPackedArray, UnsignedBitPacking};

impl BitPackedArray {
    pub fn iter_blocks<'a, T: NativePType + UnsignedBitPacking, const BLOCK_SIZE: usize>(
        &'a self,
    ) -> BitPackedArrayBlockIter<'a, T, BLOCK_SIZE>
    where
        T: NativePType + UnsignedBitPacking,
        T::UnsignedT: BitPacking,
    {
        let last_chunk_len = (self.offset() as usize + self.len()) % 1024;

        let bit_width = self.bit_width() as usize;
        let last_chunk_is_sliced = last_chunk_len > 0;

        let num_chunks = (self.offset() as usize + self.len() + 1023) / 1024;
        // println!(
        //     "first sl {}, last sl {}",
        //     first_block_is_sliced, last_chunk_is_sliced
        // );
        //
        // println!("{}, {}", num_chunks, last_chunk_len);
        // println!(
        //     "fbc {}, bc {}",
        //     num_chunks - last_chunk_is_sliced as usize,
        //     num_chunks
        // );
        // let elems_per_chunk = 128 * bit_width / size_of::<T>();

        BitPackedArrayBlockIter::<'a, T, BLOCK_SIZE> {
            packed_slice: self.packed_slice::<T::UnsignedT>(),
            block_id: 0,
            full_block_count: num_chunks - last_chunk_is_sliced as usize,
            block_count: num_chunks,
            last_chunk_len,
            offset: self.offset() as usize,
            bit_width,
            fixed_array: [T::UnsignedT::zero(); BLOCK_SIZE],
            _ph: PhantomData::default(),
        }
    }
}

impl BitPackedArray {
    pub fn dyn_iter_blocks<'a, T, const BLOCK_SIZE: usize>(
        &'a self,
    ) -> Box<dyn ArrayValueIterator<BLOCK_SIZE, Item = T> + 'a>
    where
        T: NativePType + UnsignedBitPacking,
        T::UnsignedT: BitPacking,
    {
        Box::from(self.iter_blocks::<T, BLOCK_SIZE>())
    }
}

pub struct BitPackedArrayBlockIter<'a, T, const BLOCK_SIZE: usize>
where
    T: UnsignedBitPacking,
{
    packed_slice: &'a [T::UnsignedT],
    block_id: usize,
    block_count: usize,
    full_block_count: usize,
    last_chunk_len: usize,
    bit_width: usize,
    offset: usize,
    //  TODO(joe):   let mut array: [MaybeUninit<u8>; SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
    fixed_array: [T::UnsignedT; BLOCK_SIZE],
    _ph: PhantomData<T>,
}

impl<'a, T, const BLOCK_SIZE: usize> ArrayValueIterator<BLOCK_SIZE>
    for BitPackedArrayBlockIter<'a, T, BLOCK_SIZE>
where
    T: NativePType + UnsignedBitPacking,
    T::UnsignedT: BitPacking,
{
    type Item = T;

    fn next(&mut self) -> Option<Result<&[T; BLOCK_SIZE], &[T]>> {
        let elems_per_chunk = 128 * self.bit_width / size_of::<T>();
        const FL_BLOCK: usize = 1024;
        if self.block_id >= self.block_count {
            return None;
        }
        let first_block_is_sliced = self.offset != 0;
        if self.block_id == 0 && first_block_is_sliced {
            unsafe {
                BitPacking::unchecked_unpack(
                    self.bit_width,
                    &self.packed_slice[..elems_per_chunk],
                    &mut self.fixed_array[..FL_BLOCK],
                )
            }
            self.block_id += 1;
            return Some(Err(unsafe {
                std::mem::transmute(&self.fixed_array[self.offset..FL_BLOCK])
            }));
        }
        if self.block_id + (BLOCK_SIZE / FL_BLOCK) <= self.full_block_count {
            for i in (0..BLOCK_SIZE).step_by(FL_BLOCK) {
                unsafe {
                    BitPacking::unchecked_unpack(
                        self.bit_width,
                        &self.packed_slice[self.block_id * elems_per_chunk..][..elems_per_chunk],
                        &mut self.fixed_array[i..i + FL_BLOCK],
                    )
                }
                self.block_id += 1;
            }
            Some(Ok(unsafe {
                std::mem::transmute::<&[T::UnsignedT; BLOCK_SIZE], &[T; BLOCK_SIZE]>(
                    &self.fixed_array,
                )
            }))
        } else {
            let rest = FL_BLOCK * (self.full_block_count - self.block_id) + self.last_chunk_len;
            for i in (0..rest).step_by(FL_BLOCK) {
                unsafe {
                    BitPacking::unchecked_unpack(
                        self.bit_width,
                        &self.packed_slice[self.block_id * elems_per_chunk..][..elems_per_chunk],
                        &mut self.fixed_array[i..i + FL_BLOCK],
                    )
                }
                self.block_id += 1;
            }
            Some(Err(unsafe {
                std::mem::transmute(&self.fixed_array[0..rest])
            }))
        }
    }
}

#[cfg(test)]
mod tests {

    use vortex_array::accessor::ArrayValueIterator;
    use vortex_array::arrays::PrimitiveArray;
    use vortex_array::compute::slice;

    use crate::{bitpack_encode, BitPackedArray};

    #[test]
    fn test_get_iter() {
        let mut vals = vec![1; 1024];
        vals.extend(vec![2; 1024]);
        vals.extend(vec![3; 3]);
        let prim = PrimitiveArray::from_iter(vals);
        let packed = bitpack_encode(prim, 8).unwrap();

        let mut iter = packed.iter_blocks::<i32, 2048>();
        println!(
            "{:?}",
            iter.next()
                .unwrap()
                .map(|v| (v.len(), v))
                .map_err(|e| (e.len(), e))
        );
        println!(
            "{:?}",
            iter.next()
                .unwrap()
                .map(|v| (v.len(), v))
                .map_err(|e| (e.len(), e))
        );
        println!("{:?}", iter.next());
    }

    #[test]
    fn test_get_iter2() {
        const SIZE: usize = 8192;
        let vals = vec![1; SIZE];
        let prim = PrimitiveArray::from_iter(vals);
        let packed = BitPackedArray::maybe_from(
            slice(
                &bitpack_encode(prim, 8).unwrap(),
                SIZE / (2 << 6),
                SIZE - (SIZE / 2),
            )
            .unwrap(),
        )
        .unwrap();

        let mut iter = packed.iter_blocks::<i32, 2048>();
        println!("{:?}", iter.next().unwrap().map_err(|e| (e.len(), e)));
        println!(
            "{:?}",
            iter.next()
                .unwrap()
                .map(|v| (v.len(), v))
                .map_err(|e| (e.len(), e))
        );
        println!("{:?}", iter.next().unwrap().map_err(|e| (e.len(), e)));
        println!("{:?}", iter.next());
        // println!("{:?}", iter.next());
    }

    #[test]
    fn test_get_iter_sliced() {
        let mut vals = vec![1; 1024];
        vals.extend(vec![2; 1024]);
        vals.extend(vec![3; 1024]);
        vals.extend(vec![4; 3]);
        let prim = PrimitiveArray::from_iter(vals);
        let packed =
            BitPackedArray::maybe_from(slice(&bitpack_encode(prim, 8).unwrap(), 2, 3074).unwrap())
                .unwrap();

        let mut iter = packed.iter_blocks::<i32, 2048>();
        println!("{:?}", iter.next().unwrap().map_err(|e| (e.len(), e)));
        println!(
            "{:?}",
            iter.next()
                .unwrap()
                .map(|v| (v.len(), v))
                .map_err(|e| (e.len(), e))
        );
        println!("{:?}", iter.next().unwrap().map_err(|e| (e.len(), e)));
        println!("{:?}", iter.next());
    }
}
