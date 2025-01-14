use vortex_array::iter::ArrayIterator;
use vortex_array::ArrayData;
use vortex_error::VortexResult;

use crate::streaming::StreamingCompressor;

pub struct CompressedArrayIter<'a, AS> {
    compressor: &'a StreamingCompressor<'a>,
    underlying: AS,
}

impl<'a, AS: ArrayIterator> CompressedArrayIter<'a, AS> {
    pub fn new(compressor: &'a StreamingCompressor<'a>, underlying: AS) -> Self {
        Self {
            compressor,
            underlying,
        }
    }
}

impl<'a, AS: ArrayIterator> Iterator for CompressedArrayIter<'a, AS> {
    type Item = VortexResult<ArrayData>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(nc) = self.underlying.next() {
                if let Ok(next_chunk) = nc {
                    let compressed = self.compressor.compress(&next_chunk).transpose();
                    if compressed.is_some() {
                        return compressed;
                    }
                } else {
                    return None;
                }
            } else {
                return self.compressor.flush().transpose();
            }
        }
    }
}
