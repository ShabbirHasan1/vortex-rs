use std::ops::Deref;
use std::sync::{Arc, RwLock};

use vortex_buffer::ByteBuffer;
use vortex_dtype::DType;
use vortex_error::{vortex_bail, VortexResult};

use crate::compute::FilterMask;
use crate::data::InnerArrayData;
use crate::encoding::EncodingRef;
use crate::stats::StatsSet;
use crate::{ArrayDType, ArrayData, ArrayMetadata};

/// Owned [`ArrayData`] with serialized metadata, backed by heap-allocated memory.
#[derive(Debug)]
pub(super) struct OwnedArrayData {
    pub(super) encoding: EncodingRef,
    pub(super) dtype: DType,
    /// A lazy filter mask that defines the array length, maybe false for an array element has an
    /// undefined value.
    pub(super) mask: FilterMask,
    pub(super) metadata: Arc<dyn ArrayMetadata>,
    pub(super) buffers: Option<Box<[ByteBuffer]>>,
    pub(super) children: Option<Box<[ArrayData]>>,
    pub(super) stats_set: RwLock<StatsSet>,
    #[cfg(feature = "canonical_counter")]
    pub(super) canonical_counter: std::sync::atomic::AtomicUsize,
}

impl OwnedArrayData {
    pub fn metadata(&self) -> &Arc<dyn ArrayMetadata> {
        &self.metadata
    }

    pub fn byte_buffer(&self, index: usize) -> Option<&ByteBuffer> {
        self.buffers.as_ref().and_then(|b| b.get(index))
    }

    // Applies a new filter mask to the array, unsafely ignoring the current mask, since filtered
    // values are undefined.
    // This is safe if:
    //   - length of both masks must be equal.
    //   - The new mask (`mask`) can be defined as a bitand `self.mask` and another mask `other`
    //         `mask` == `self.mask`.bitand(`other`)
    pub fn with_selection(&self, mask: FilterMask) -> InnerArrayData {
        assert_eq!(self.mask.len(), mask.len());
        // TODO(joe): optimize
        InnerArrayData::Owned(Arc::new(OwnedArrayData {
            encoding: self.encoding,
            dtype: self.dtype.clone(),
            mask,
            metadata: self.metadata.clone(),
            buffers: self.buffers.clone(),
            children: self.children.clone(),
            // TODO(joe): handle poisoning
            stats_set: RwLock::new(self.stats_set.read().unwrap().deref().clone()),
            #[cfg(feature = "canonical_counter")]
            canonical_counter: *self.canonical_counter,
        }))
    }

    // We want to allow these panics because they are indicative of implementation error.
    #[allow(clippy::panic_in_result_fn)]
    pub fn child(&self, index: usize, dtype: &DType, mask: &FilterMask) -> VortexResult<ArrayData> {
        match self.children.as_ref().and_then(|c| c.get(index)) {
            None => vortex_bail!(
                "ArrayData::child({}): child {index} not found",
                self.encoding.id().as_ref()
            ),
            Some(child) => {
                assert_eq!(
                    child.dtype(),
                    dtype,
                    "child {index} requested with incorrect dtype for encoding {}",
                    self.encoding.id().as_ref(),
                );
                assert_eq!(
                    child.len(),
                    mask.len(),
                    "child {index} requested with incorrect length for encoding {}",
                    self.encoding.id().as_ref(),
                );
                if mask.keep_all() {
                    Ok(child.clone())
                } else {
                    let mut child = child.clone();
                    child.with_selection(self.mask.intersect_by_rank(mask))?;
                    Ok(child)
                }
            }
        }
    }

    pub fn nchildren(&self) -> usize {
        self.children.as_ref().map_or(0, |c| c.len())
    }
}
