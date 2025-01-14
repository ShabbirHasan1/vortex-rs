use std::cell::RefCell;
use std::mem;

use vortex_array::{ArrayData, IntoArrayData};
use vortex_dict::DictArray;
use vortex_error::{VortexExpect, VortexResult};

use crate::streaming::{EncodingCompressor, StreamingCompressor};

pub struct DictCompressor {
    current_dictionary: RefCell<Option<ArrayData>>,
    current_indices: Vec<ArrayData>,
}

impl DictCompressor {
    pub fn new() -> Self {
        Self {
            current_dictionary: RefCell::new(None),
            current_indices: Vec::new(),
        }
    }
}

impl EncodingCompressor for DictCompressor {
    fn maybe_compress<'a>(
        &'a mut self,
        _array: &ArrayData,
        _compressor: &'a StreamingCompressor,
    ) -> VortexResult<Option<ArrayData>> {
        if let Some(_dict) = self.current_dictionary.take() {
            todo!()
        } else {
            todo!()
        }
    }

    fn flush(&mut self) -> VortexResult<Option<ArrayData>> {
        let indices = mem::take(&mut self.current_indices);
        if !indices.is_empty() {
            DictArray::try_new(
                ArrayData::try_from(indices)?,
                self.current_dictionary
                    .take()
                    .vortex_expect("Must have a dictionary if there's indices"),
            )
            .map(|a| a.into_array())
            .map(Some)
        } else {
            Ok(None)
        }
    }
}
