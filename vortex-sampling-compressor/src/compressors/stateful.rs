use vortex_array::aliases::hash_set::HashSet;
use vortex_array::{Array, EncodingId};
use vortex_error::VortexResult;

use crate::compressors::CompressionTree;
use crate::SamplingCompressor;

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
    ) -> VortexResult<O>;

    fn finish(&self) -> VortexResult<O>;
}
