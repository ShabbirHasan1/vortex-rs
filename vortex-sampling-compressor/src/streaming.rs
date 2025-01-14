use std::cell::RefCell;

use vortex_array::aliases::hash_set::HashSet;
use vortex_array::iter::ArrayIterator;
use vortex_array::ArrayData;
use vortex_error::VortexResult;

use crate::iter::CompressedArrayIter;

pub trait EncodingStrategy {
    fn can_compress(&self, array: &ArrayData) -> Option<Box<dyn EncodingCompressor>>;
}

pub trait EncodingCompressor {
    fn maybe_compress<'a>(
        &'a mut self,
        array: &ArrayData,
        compressor: &'a StreamingCompressor,
    ) -> VortexResult<Option<ArrayData>>;

    fn flush(&mut self) -> VortexResult<Option<ArrayData>>;
}

pub struct StreamingCompressor<'a> {
    strategies: HashSet<&'a dyn EncodingStrategy>,
    current_compressor: RefCell<Option<Box<dyn EncodingCompressor>>>,
}

impl<'a> StreamingCompressor<'a> {
    pub fn compress(&self, _array: &ArrayData) -> VortexResult<Option<ArrayData>> {
        let _strategy = self.strategies.iter().next().unwrap();

        todo!();
        // let mut compressor = strategy.can_compress(array);
        // if let Some(child) = compressor.maybe_compress(array, self)? {
        //     self.current_compressor.replace(Some(compressor));
        //     Ok(Some(child))
        // } else {
        //     self.current_compressor.replace(None);
        //     // compress with fallback
        //     todo!()
        // }
    }

    pub fn flush(&self) -> VortexResult<Option<ArrayData>> {
        let res = self
            .current_compressor
            .take()
            .and_then(|mut s| s.flush().transpose());
        self.current_compressor.replace(None);
        res.transpose()
    }

    pub fn compress_streaming<T: ArrayIterator>(
        &'a self,
        array_stream: T,
    ) -> impl Iterator<Item=VortexResult<ArrayData>> + use < 'a, T > {
        CompressedArrayIter::<'a, T>::new(self, array_stream)
    }
}
