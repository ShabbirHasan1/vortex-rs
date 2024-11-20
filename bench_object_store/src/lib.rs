#![allow(clippy::unwrap_used)]

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::StreamExt;
use log::trace;
use vortex::error::VortexResult;
use vortex::ArrayData;

pub mod s3;

pub type ScanFuture = BoxFuture<'static, BoxStream<'static, VortexResult<ArrayData>>>;

pub trait Scan {
    fn open(&self, path: &str) -> ScanFuture;
}

impl Scan for Box<dyn Scan> {
    fn open(&self, path: &str) -> ScanFuture {
        self.as_ref().open(path)
    }
}

impl<S: Scan + Sized> Scan for Box<S> {
    fn open(&self, path: &str) -> ScanFuture {
        S::open(self.as_ref(), path)
    }
}

pub struct RunScan<S> {
    inner: S,
    path: String,
}

impl<S> RunScan<S> {
    pub fn new(inner: S, path: String) -> Self {
        Self { inner, path }
    }
}

impl<S: Scan> RunScan<S> {
    /// Return a set of metrics from doing a full-scan, as well as per-batch metrics.
    pub async fn full_scan(&self) {
        self.inner
            .open(self.path.as_str())
            .await
            .for_each(|batch| async move {
                let batch = batch.unwrap();
                trace!(
                    "received batch with len = {} size = {}B",
                    batch.len(),
                    batch.nbytes()
                );
            })
            .await;
    }
}
