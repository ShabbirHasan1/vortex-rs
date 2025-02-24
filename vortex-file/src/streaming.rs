#![allow(dead_code)]

use std::io::Read;
use std::marker::PhantomData;
use std::sync::Arc;

use futures::stream::BoxStream;
use futures::{stream, Stream};
use vortex_error::VortexResult;
use vortex_io::VortexReadAt;
use vortex_layout::scan::ScanDriver;
use vortex_layout::segments::AsyncSegmentReader;
use vortex_metrics::VortexMetrics;

use crate::segments::SegmentCache;
use crate::{FileLayout, FileType, VortexOpenOptions};

/// A type of Vortex file that consumes I/O with a pool of streaming file handles, rather than
/// range-based coalescing.
pub struct StreamingVortexFile<R>(PhantomData<R>);

pub trait StreamingRead: VortexReadAt {
    fn stream_from(&self, offset: u64) -> Box<dyn Read>;
}

impl<R: StreamingRead> VortexOpenOptions<StreamingVortexFile<R>> {
    /// Open an in-memory file contained in the provided buffer.
    pub fn streaming(read: R) -> Self {
        Self::new(read, ())
    }
}

impl<R: StreamingRead> FileType for StreamingVortexFile<R> {
    type Options = ();
    type Read = R;
    type ScanDriver = StreamingScanDriver<R>;

    fn scan_driver(
        read: Self::Read,
        _options: Self::Options,
        file_layout: FileLayout,
        segment_cache: Arc<dyn SegmentCache>,
        metrics: VortexMetrics,
    ) -> Self::ScanDriver {
        StreamingScanDriver {
            read,
            file_layout,
            segment_cache,
            metrics: metrics.into(),
        }
    }
}

pub struct StreamingScanDriver<R> {
    read: R,
    file_layout: FileLayout,
    segment_cache: Arc<dyn SegmentCache>,
    metrics: VortexMetrics,
}

impl<R> StreamingScanDriver<R> {
    fn new(
        read: R,
        file_layout: FileLayout,
        segment_cache: Arc<dyn SegmentCache>,
        metrics: VortexMetrics,
    ) -> Self {
        // What we can do, is traverse the FileLayout and create a mapping of segment -> column.
        // That way we can know that a column typically reads in increasing row order, and
        // therefore when we should re-use an existing request.

        Self {
            read,
            file_layout,
            segment_cache,
            metrics,
        }
    }
}

impl<R: StreamingRead> ScanDriver for StreamingScanDriver<R> {
    fn segment_reader(&self) -> Arc<dyn AsyncSegmentReader> {
        todo!()
    }

    fn io_stream(self) -> impl Stream<Item = VortexResult<()>> + 'static {
        stream::repeat_with(|| Ok(()))
    }
}
