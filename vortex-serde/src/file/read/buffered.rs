use std::collections::VecDeque;
use std::mem;
use std::sync::Arc;

use vortex_array::aliases::hash_set::HashSet;
use vortex_array::array::{ChunkedArray, NullArray, StructArray};
use vortex_array::compute::unary::scalar_at;
use vortex_array::{Array, ArrayDType, IntoArray};
use vortex_dtype::FieldNames;
use vortex_error::{vortex_bail, VortexExpect as _, VortexResult};
use vortex_scalar::BoolScalar;

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
    layouts: VecDeque<(usize, RangedLayoutReader)>,
    arrays: Vec<Array>,
    n_chunks: usize,
    scan: Scan,
    chunk_mask: Option<Array>,
}

#[derive(Debug)]
pub enum MetadataReader {
    NoMetadata,
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
            layouts: layouts.into_iter().enumerate().collect::<VecDeque<_>>(),
            arrays: Vec::new(),
            n_chunks,
            scan,
            chunk_mask: None,
        }
    }

    // TODO(robert): Support out of order reads
    fn buffer_read(&mut self, mask: &RowMask) -> VortexResult<Option<Vec<Message>>> {
        if self.chunk_mask.is_none() {
            println!("BufferedLayoutReader: No chunk_mask");
            let metadata = match mem::take(&mut self.metadata_reader) {
                // FIXME(DK): pull this out
                metadata_reader @ Some(MetadataReader::NoMetadata) => {
                    self.metadata_reader = metadata_reader;
                    None
                }
                Some(MetadataReader::NotYetRead(mut reader)) => {
                    match reader.read_selection(&RowMask::new_valid_between(0, self.n_chunks))? {
                        Some(BatchRead::ReadMore(messages)) => {
                            println!("BufferedLayoutReader: No chunk_mask: need to read more");
                            self.metadata_reader = Some(MetadataReader::NotYetRead(reader));
                            return Ok(Some(messages));
                        }
                        Some(BatchRead::Batch(array)) => {
                            println!("BufferedLayoutReader: No chunk_mask: read an array");
                            self.metadata_reader = Some(MetadataReader::Read(array.clone()));
                            Some(array)
                        }
                        None => {
                            vortex_bail!("unexpected end of stream while reading metadata array")
                        }
                    }
                }
                Some(MetadataReader::Read(array)) => {
                    println!("BufferedLayoutReader: No chunk_mask: already read an array");
                    self.metadata_reader = Some(MetadataReader::Read(array.clone()));
                    Some(array)
                }
                None => vortex_bail!("Called buffer_read while buffer_read was running"),
            };

            println!(
                "BufferedLayoutReader: No chunk_mask: scan.expr={}, metadata={}",
                self.scan
                    .expr
                    .as_ref()
                    .map(|x| format!("{}", x))
                    .unwrap_or_else(|| "None".to_string()),
                metadata
                    .as_ref()
                    .map(|x| x.pretty())
                    .unwrap_or_else(|| "None".to_string())
            );
            self.chunk_mask = self
                .scan
                .expr
                .as_ref()
                .zip(metadata)
                .and_then(|(expression, metadata)| {
                    println!("PruningPreciate: original_expr:{}", expression);
                    let predicate = PruningPredicate::try_new(expression)?;
                    println!(
                        "PruningPreciate: predicate={} original_expr:{}",
                        predicate, expression
                    );
                    Some((predicate, metadata))
                })
                .map(|(predicate, metadata)| {
                    metadata.with_dyn(|x| {
                        let logical_validity = x.logical_validity();
                        let metadata = x
                            .as_struct_array()
                            .vortex_expect("metadata must be struct array");
                        let required_field_names = predicate.required_stat_field_names();
                        let dtype = metadata.struct_dtype();
                        let known_names: HashSet<String> =
                            dtype.names().iter().map(|x| x.to_string()).collect();
                        let missing_names: Vec<String> = required_field_names
                            .difference(&known_names)
                            .cloned()
                            .collect();
                        let n_missing = missing_names.len();
                        let null_filled_field_names: FieldNames = Arc::from(
                            dtype
                                .names()
                                .iter()
                                .cloned()
                                .chain(missing_names.into_iter().map(Arc::from))
                                .collect::<Vec<_>>(),
                        );
                        let null_filled_metadata = StructArray::try_new(
                            null_filled_field_names,
                            (0..metadata.nfields())
                                .map(|index| {
                                    metadata
                                        .field(index)
                                        .vortex_expect("array must have as many fields as its type")
                                })
                                .chain(
                                    (0..n_missing)
                                        .map(|_| NullArray::new(metadata.len()).into_array()),
                                )
                                .collect(),
                            metadata.len(),
                            logical_validity.into_validity(),
                        )?
                        .into_array();
                        predicate.expr().evaluate(&null_filled_metadata)
                    })
                })
                .transpose()?
        }
        println!(
            "BufferedLayoutReader: chunk_mask={}",
            self.chunk_mask
                .as_ref()
                .map(|x| x.pretty())
                .unwrap_or_else(|| "None".to_string())
        );

        // FIXME(DK): convert the pruner array to a boolean, get an iterator and zip it with the
        // children to determine if we should keep that split
        //
        // Maybe we actually stash the mask in MetadataReader? The index of interest is self.arrays.len().

        while let Some((index, ((begin, end), layout))) = self.layouts.pop_front() {
            if mask.begin() <= begin && begin < mask.end()
                || mask.begin() < end && end <= mask.end()
            {
                self.layouts.push_front((index, ((begin, end), layout)));
                break;
            }
        }

        while let Some((index, ((begin, end), mut layout))) = self.layouts.pop_front() {
            // This selection doesn't know about rows in this chunk, we should put it back and wait for another request with different range
            if mask.end() <= begin || mask.begin() > end {
                self.layouts.push_front((index, ((begin, end), layout)));
                return Ok(None);
            }

            let chunk_is_pruned = self
                .chunk_mask
                .as_ref()
                .map(|chunk_mask| -> VortexResult<_> {
                    Ok(BoolScalar::try_from(&scalar_at(chunk_mask, index)?)?
                        .value()
                        .map(|x| {
                            // assert!(!x);
                            x
                        })
                        // FIXME(DK): what does a null in the array mean
                        .unwrap_or(false))
                })
                .transpose()?
                .unwrap_or(false);

            println!(
                "BufferedLayoutReader: chunk_is_pruned={}, index={}",
                chunk_is_pruned, index,
            );

            if chunk_is_pruned {
                // do not push the layout back as it is pruned.
                return Ok(None);
            }

            let layout_selection = mask.slice(begin, end).shift(begin)?;
            if let Some(rr) = layout.read_selection(&layout_selection)? {
                match rr {
                    BatchRead::ReadMore(m) => {
                        self.layouts.push_front((index, ((begin, end), layout)));
                        return Ok(Some(m));
                    }
                    BatchRead::Batch(a) => {
                        self.arrays.push(a);
                        if end > mask.end() {
                            self.layouts.push_front((index, ((begin, end), layout)));
                            return Ok(None);
                        }
                    }
                }
            } else {
                if end > mask.end() && begin < mask.end() {
                    self.layouts.push_front((index, ((begin, end), layout)));
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
