use std::iter;

use vortex_dtype::NativePType;
use vortex_error::VortexResult;

use crate::accessor::{ArrayAccessor, ArrayValueIterator};
use crate::arrays::primitive::PrimitiveArray;
use crate::validity::Validity;
use crate::IntoArrayVariant;

impl<T: NativePType> ArrayAccessor<T> for PrimitiveArray {
    fn with_iterator<F, R>(&self, f: F) -> VortexResult<R>
    where
        F: for<'a> FnOnce(&mut (dyn Iterator<Item = Option<&'a T>>)) -> R,
    {
        match self.validity() {
            Validity::NonNullable | Validity::AllValid => {
                let mut iter = self.as_slice::<T>().iter().map(Some);
                Ok(f(&mut iter))
            }
            Validity::AllInvalid => Ok(f(&mut iter::repeat_n(None, self.len()))),
            Validity::Array(v) => {
                let validity = v.into_bool()?.boolean_buffer();
                let mut iter = self
                    .as_slice::<T>()
                    .iter()
                    .zip(validity.iter())
                    .map(|(value, valid)| valid.then_some(value));
                Ok(f(&mut iter))
            }
        }
    }
}

impl PrimitiveArray {
    pub fn iter_blocks<'a, T: NativePType, const BLOCK_SIZE: usize>(
        &'a self,
    ) -> PrimitiveArrayBlockIter<'a, T, BLOCK_SIZE> {
        PrimitiveArrayBlockIter {
            slice: self.as_slice::<T>(),
            pos: 0,
            // fixed_array: [MaybeUninit::uninit(); BLOCK_SIZE],
        }
    }
}

pub struct PrimitiveArrayBlockIter<'a, T, const BLOCK_SIZE: usize> {
    slice: &'a [T],
    pos: usize,
    //     let mut array: [MaybeUninit<u8>; SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
    // fixed_array: [MaybeUninit<T>; BLOCK_SIZE],
}

impl<'a, T: NativePType, const BLOCK_SIZE: usize> ArrayValueIterator<BLOCK_SIZE>
    for PrimitiveArrayBlockIter<'a, T, BLOCK_SIZE>
{
    type Item = T;

    fn next(&mut self) -> Option<Result<&[T; BLOCK_SIZE], &[T]>> {
        if self.pos >= self.slice.len() {
            return None;
        }
        if self.pos + BLOCK_SIZE < self.slice.len() {
            let res = &self.slice[self.pos..self.pos + BLOCK_SIZE];
            self.pos += BLOCK_SIZE;
            Some(Ok(unsafe {
                &*(res as *const [T] as *const [T; BLOCK_SIZE])
            }))
        } else {
            let pos = self.pos;
            self.pos = self.slice.len();
            Some(Err(&self.slice[pos..]))
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::accessor::ArrayValueIterator;
    use crate::arrays::PrimitiveArray;

    #[test]
    fn test_get_iter() {
        let prim = PrimitiveArray::from_iter(0..1025);

        let mut res = prim.iter_blocks::<i32>();
        println!("{:?}", res.next());
        println!("{:?}", res.next());
    }
}
