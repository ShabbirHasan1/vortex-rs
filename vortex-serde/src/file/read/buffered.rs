use std::collections::VecDeque;
use std::mem;

use vortex_array::array::ChunkedArray;
use vortex_array::{Array, ArrayDType, IntoArray};
use vortex_error::{vortex_bail, VortexResult};

use super::Scan;
use crate::file::pruning::PruningPredicate;
use crate::file::read::mask::RowMask;
use crate::file::read::{BatchRead, LayoutReader};
use crate::file::Message;

pub type RangedLayoutReader = ((usize, usize), Box<dyn LayoutReader>);

/// Layout reader that continues reading children until all rows referenced in the mask have been handled
#[derive(Debug)]
pub struct BufferedLayoutReader {
    metadata_reader: Option<MetadataReader>,
    layouts: VecDeque<RangedLayoutReader>,
    arrays: Vec<Array>,
    n_chunks: usize,
    scan: Scan,
}

#[derive(Debug)]
pub enum MetadataReader {
    NotYetRead(Box<dyn LayoutReader>),
    Read(Array),
}

impl BufferedLayoutReader {
    pub fn new(
        metadata_reader: MetadataReader,
        layouts: VecDeque<RangedLayoutReader>,
        scan: Scan,
    ) -> Self {
        let n_chunks = layouts.len();
        Self {
            metadata_reader: Some(metadata_reader),
            layouts,
            arrays: Vec::new(),
            n_chunks,
            scan,
        }
    }

    // TODO(robert): Support out of order reads
    fn buffer_read(&mut self, mask: &RowMask) -> VortexResult<Option<Vec<Message>>> {
        let metadata = match mem::take(&mut self.metadata_reader) {
            // FIXME(DK): pull this out
            Some(MetadataReader::NotYetRead(mut reader)) => {
                match reader.read_selection(&RowMask::new_valid_between(0, self.n_chunks))? {
                    Some(BatchRead::ReadMore(messages)) => return Ok(Some(messages)),
                    Some(BatchRead::Batch(array)) => {
                        self.metadata_reader = Some(MetadataReader::Read(array.clone()));
                        array
                    }
                    None => vortex_bail!("unexpected end of stream while reading metadata array"),
                }
            }
            Some(MetadataReader::Read(array)) => {
                self.metadata_reader = Some(MetadataReader::Read(array.clone()));
                array
            }
            None => vortex_bail!("Called buffer_read while buffer_read was running"),
        };

        let pruner = self
            .scan
            .expr
            .as_ref()
            .map(PruningPredicate::try_new)
            .flatten()
            .map(|pruner| pruner.expr().evaluate(&metadata))
            .transpose()?;

        // FIXME(DK): convert the pruner array to a boolean, get an iterator and zip it with the
        // children to determine if we should keep that split
        //
        // Maybe we actually stash the mask in MetadataReader? The index of interest is self.arrays.len().

        while let Some(((begin, end), layout)) = self.layouts.pop_front() {
            if mask.begin() <= begin && begin < mask.end()
                || mask.begin() < end && end <= mask.end()
            {
                self.layouts.push_front(((begin, end), layout));
                break;
            }
        }

        while let Some(((begin, end), mut layout)) = self.layouts.pop_front() {
            // This selection doesn't know about rows in this chunk, we should put it back and wait for another request with different range
            if mask.end() <= begin || mask.begin() > end {
                self.layouts.push_front(((begin, end), layout));
                return Ok(None);
            }
            let layout_selection = mask.slice(begin, end).shift(begin)?;
            if let Some(rr) = layout.read_selection(&layout_selection)? {
                match rr {
                    BatchRead::ReadMore(m) => {
                        self.layouts.push_front(((begin, end), layout));
                        return Ok(Some(m));
                    }
                    BatchRead::Batch(a) => {
                        self.arrays.push(a);
                        if end > mask.end() {
                            self.layouts.push_front(((begin, end), layout));
                            return Ok(None);
                        }
                    }
                }
            } else {
                if end > mask.end() && begin < mask.end() {
                    self.layouts.push_front(((begin, end), layout));
                    return Ok(None);
                }
                continue;
            }
        }
        Ok(None)
    }

    pub fn read_next(&mut self, mask: &RowMask) -> VortexResult<Option<BatchRead>> {
        if let Some(bufs) = self.buffer_read(mask)? {
            return Ok(Some(BatchRead::ReadMore(bufs)));
        }

        let mut result = mem::take(&mut self.arrays);
        match result.len() {
            0 | 1 => Ok(result.pop().map(BatchRead::Batch)),
            _ => {
                let dtype = result[0].dtype().clone();
                Ok(Some(BatchRead::Batch(
                    ChunkedArray::try_new(result, dtype)?.into_array(),
                )))
            }
        }
    }
}
