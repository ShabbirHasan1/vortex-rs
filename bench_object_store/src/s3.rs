use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use object_store::aws::AmazonS3Builder;
use object_store::path::Path;
use object_store::ObjectStore;
use vortex::file::{LayoutContext, LayoutDeserializer, VortexReadBuilder};
use vortex::io::ObjectStoreReadAt;
use vortex::sampling_compressor::ALL_ENCODINGS_CONTEXT;

use crate::{Scan, ScanFuture};

pub struct S3Scan {
    object_store: Arc<dyn ObjectStore>,
}

impl S3Scan {
    pub fn new(bucket: impl AsRef<str>) -> Self {
        let object_store = AmazonS3Builder::from_env()
            .with_bucket_name(bucket.as_ref())
            .build()
            .unwrap();

        Self {
            object_store: Arc::new(object_store),
        }
    }
}

impl Scan for S3Scan {
    fn open(&self, path: &str) -> ScanFuture {
        // We should consider implement some sort of expression support. This might be
        // a parser that we can read from instead...fuck.
        // We don't have a way to build these expressions except via Rust code.
        let reader = ObjectStoreReadAt::new(self.object_store.clone(), Path::from(path));
        VortexReadBuilder::new(
            reader,
            LayoutDeserializer::new(
                ALL_ENCODINGS_CONTEXT.clone(),
                Arc::new(LayoutContext::default()),
            ),
        )
        .build()
        .map(|fut| fut.unwrap().boxed())
        .boxed()
    }
}
