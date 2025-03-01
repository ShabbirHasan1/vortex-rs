use std::cell::LazyCell;

use vortex_array::serde::ArrayParts;
use vortex_array::stats::{ArrayStats, StatsSetRef};
use vortex_array::vtable::VTableRef;
use vortex_array::{
    ArrayCanonicalImpl, ArrayImpl, ArrayRef, ArrayStatisticsImpl, ArrayValidityImpl,
    ArrayVariantsImpl, ArrayVisitorImpl, Canonical, EmptyMetadata,
};
use vortex_dtype::DType;
use vortex_error::VortexResult;
use vortex_mask::Mask;
use wasmtime::Instance;

use crate::v1::WasmEncoding;

#[derive(Clone, Debug)]
pub struct WasmArray {
    pub(crate) dtype: DType,
    pub(crate) len: usize,
    pub(crate) parts: ArrayParts,
    pub(crate) stats_set: ArrayStats,

    pub(crate) vtable: VTableRef,
    pub(crate) decoded: LazyCell<ArrayRef>,
    pub(crate) instance: Instance,
}

impl ArrayImpl for WasmArray {
    type Encoding = WasmEncoding;

    fn _len(&self) -> usize {
        self.len
    }

    fn _dtype(&self) -> &DType {
        &self.dtype
    }

    fn _vtable(&self) -> VTableRef {
        self.vtable.clone()
    }
}

impl ArrayCanonicalImpl for WasmArray {
    fn _to_canonical(&self) -> VortexResult<Canonical> {
        todo!()
    }
}

impl ArrayStatisticsImpl for WasmArray {
    fn _stats_ref(&self) -> StatsSetRef<'_> {
        todo!()
    }
}

impl ArrayValidityImpl for WasmArray {
    fn _is_valid(&self, _index: usize) -> VortexResult<bool> {
        todo!()
    }

    fn _all_valid(&self) -> VortexResult<bool> {
        todo!()
    }

    fn _all_invalid(&self) -> VortexResult<bool> {
        todo!()
    }

    fn _validity_mask(&self) -> VortexResult<Mask> {
        todo!()
    }
}

impl ArrayVariantsImpl for WasmArray {}

impl ArrayVisitorImpl for WasmArray {
    fn _metadata(&self) -> EmptyMetadata {
        EmptyMetadata
    }
}
