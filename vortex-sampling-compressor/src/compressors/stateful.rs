use std::marker::PhantomData;

use vortex_array::aliases::hash_set::HashSet;
use vortex_array::{Array, EncodingId};
use vortex_error::VortexResult;

use crate::compressors::CompressionTree;
use crate::SamplingCompressor;

pub struct DictCompressionStrategy;

pub trait DictWriter<O> {
    fn write_values(&self, values: Array) -> VortexResult<()>;
    fn write_codes(&self, codes: Array) -> VortexResult<Option<O>>;
}

pub struct DictCompressor<O, W: DictWriter<O>> {
    writer: W,
    _phantom: PhantomData<O>,
}

impl<O> EncodingComp<O> for DictCompressionStrategy {
    fn id(&self) -> &str {
        todo!()
    }

    fn cost(&self) -> u8 {
        todo!()
    }

    fn can_compress(&self, array: &Array) -> Option<&dyn StatefulCompressor<O>> {
        todo!()
    }

    fn used_encodings(&self) -> HashSet<EncodingId> {
        todo!()
    }
}

pub trait EncodingComp<O> {
    fn id(&self) -> &str;

    fn cost(&self) -> u8;

    fn can_compress(&self, array: &Array) -> Option<&dyn StatefulCompressor<O>>;

    fn used_encodings(&self) -> HashSet<EncodingId>;
}

pub trait StatefulCompressor<O> {
    fn compress<'a>(
        &'a self,
        array: &Array,
        like: Option<CompressionTree<'a>>,
        ctx: SamplingCompressor<'a>,
    ) -> VortexResult<Option<O>>;

    fn finish(&self) -> VortexResult<Option<O>>;
}

impl<O, W: DictWriter<O>> StatefulCompressor<O> for DictCompressor<O, W> {
    fn compress<'a>(
        &'a self,
        array: &Array,
        like: Option<CompressionTree<'a>>,
        ctx: SamplingCompressor<'a>,
    ) -> VortexResult<Option<O>> {
        todo!()
    }

    fn finish(&self) -> VortexResult<Option<O>> {
        todo!()
    }
}
