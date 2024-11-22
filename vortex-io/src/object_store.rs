use std::io;
use std::ops::Range;
use std::sync::Arc;

use bytes::Bytes;
use object_store::path::Path;
use object_store::{MultipartUpload, ObjectStore, PutPayload};
use vortex_error::{VortexExpect, VortexResult};

use crate::{IoBuf, VortexReadAt, VortexWrite};
// use bytes::Bytes;
// use futures_util::StreamExt;
// use object_store::path::Path;
// use object_store::{GetOptions, GetRange, ObjectStore, WriteMultipart};
// use vortex_buffer::io_buf::IoBuf;
// use vortex_buffer::Buffer;
// use vortex_error::{vortex_panic, VortexError, VortexResult};

// use crate::{VortexBufReader, VortexReadAt, VortexWrite, BUFFER_ALIGNMENT};

// pub trait ObjectStoreExt {
//     fn vortex_read(
//         &self,
//         location: &Path,
//         range: Range<usize>,
//     ) -> impl Future<Output = VortexResult<VortexBufReader<impl VortexReadAt>>>;

//     fn vortex_reader(&self, location: &Path) -> impl VortexReadAt;

//     fn vortex_writer(
//         &self,
//         location: &Path,
//     ) -> impl Future<Output = VortexResult<impl VortexWrite>>;
// }

// impl ObjectStoreExt for Arc<dyn ObjectStore> {
//     async fn vortex_read(
//         &self,
//         location: &Path,
//         range: Range<usize>,
//     ) -> VortexResult<VortexBufReader<impl VortexReadAt>> {
//         let bytes = self.get_range(location, range).await?;
//         Ok(VortexBufReader::new(Buffer::from(bytes)))
//     }

//     fn vortex_reader(&self, location: &Path) -> impl VortexReadAt {
//         ObjectStoreReadAt::new(self.clone(), location.clone())
//     }

//     async fn vortex_writer(&self, location: &Path) -> VortexResult<impl VortexWrite> {
//         Ok(ObjectStoreWriter::new(WriteMultipart::new_with_chunk_size(
//             self.put_multipart(location).await?,
//             10 * 1024 * 1024,
//         )))
//     }
// }

#[derive(Clone)]
pub struct ObjectStoreReadAt {
    object_store: Arc<dyn ObjectStore>,
    location: Path,
}

impl ObjectStoreReadAt {
    pub fn new(object_store: Arc<dyn ObjectStore>, location: Path) -> Self {
        Self {
            object_store,
            location,
        }
    }
}

impl VortexReadAt for ObjectStoreReadAt {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    async fn read_byte_range(&self, range: Range<u64>) -> io::Result<Bytes> {
        let object_store = self.object_store.clone();
        let location = self.location.clone();

        let start = usize::try_from(range.start).vortex_expect("range.start");
        let end = usize::try_from(range.end).vortex_expect("range.end");
        object_store
            .get_range(&location, start..end)
            .await
            .map_err(Into::into)
// =======
//         Box::pin(async move {
//             let read_start: usize = pos.try_into().vortex_expect("pos");
//             let read_end: usize = (pos + len).try_into().vortex_expect("pos + len");
//             let len: usize = len.try_into().vortex_expect("len does not fit into usize");
//             // =======
//             //             let start_range = pos as usize;

//             //             // Allocate an aligned vector to read into.
//             //             //
//             //             // This is
//             //             let alloc_size =
//             //                 ((len as usize) + (BUFFER_ALIGNMENT - 1)).next_multiple_of(BUFFER_ALIGNMENT);
//             //             let mut buf = Vec::<u8>::with_capacity(alloc_size as _);
//             //             let padding = buf.as_ptr().align_offset(BUFFER_ALIGNMENT);
//             //             unsafe { buf.set_len(padding) };

//             //             let get_range = start_range..(start_range + len as usize);
//             // >>>>>>> 68d20e8a3 (fix: allocate aligned buffers when reading through ObjectStore)

//             let response = object_store
//                 .get_opts(
//                     &location,
//                     GetOptions {
//                         range: Some(GetRange::Bounded(read_start..read_end)),
//                         // =======
//                         //                         range: Some(GetRange::Bounded(get_range)),
//                         // >>>>>>> 68d20e8a3 (fix: allocate aligned buffers when reading through ObjectStore)
//                         ..Default::default()
//                     },
//                 )
//                 .await?;

//             // NOTE: ObjectStore specializes the payload based on if it is backed by a File or if
//             //  it's coming from a network stream. Internally they optimize the File implementation
//             //  to only perform a single allocation when calling `.bytes().await`, which we
//             //  replicate here by emitting the contents directly into our aligned buffer.
//             let mut buffer = BytesMut::with_capacity(len);
//             match response.payload {
//                 GetResultPayload::File(file, _) => {
//                     unsafe { buffer.set_len(len) };
//                     file.read_exact_at(&mut buffer, pos)?;
//                 }
//                 GetResultPayload::Stream(mut byte_stream) => {
//                     while let Some(bytes) = byte_stream.next().await {
//                         buffer.extend_from_slice(&bytes?);
//                     }
//                 }
//             }
//             Ok(buffer.freeze())
//             // =======
//             //             let mut byte_stream = response.into_stream();
//             //             while let Some(bytes) = byte_stream.next().await {
//             //                 let bytes = bytes?;
//             //                 buf.extend_from_slice(&bytes);
//             //             }

//             //             // bytes_unaligned will contain the entire allocation, so that on Drop the entire buf
//             //             // is freed.
//             //             //
//             //             // bytes_unaligned is a sliced view on top of bytes_unaligned.
//             //             //
//             //             // bytes_aligned
//             //             //     | parent    \  *ptr
//             //             //     v            |
//             //             // bytes_unaligned  |
//             //             //     |            |
//             //             //     | *ptr       |
//             //             //     v            v
//             //             //     +------------+------------------+----------------+
//             //             //     | padding    |   content        | spare capacity |
//             //             //     +------------+------------------+----------------+
//             //             //
//             //             let bytes_unaligned = Bytes::from(buf);
//             //             let bytes_aligned = bytes_unaligned.slice(padding..);

//             //             assert_eq!(
//             //                 bytes_aligned.as_ptr().align_offset(BUFFER_ALIGNMENT),
//             //                 0,
//             //                 "buffer must be 64-byte aligned"
//             //             );
//             //             Ok(bytes_aligned)
//             // >>>>>>> 68d20e8a3 (fix: allocate aligned buffers when reading through ObjectStore)
//         })
// >>>>>>> b2dc08e2f (fix: allocate aligned buffers when reading through ObjectStore)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    async fn size(&self) -> io::Result<u64> {
        let object_store = self.object_store.clone();
        let location = self.location.clone();
        Ok(object_store.head(&location).await?.size as u64)
    }
}

pub struct ObjectStoreWriter {
    upload: Box<dyn MultipartUpload>,
}

impl ObjectStoreWriter {
    pub async fn new(object_store: Arc<dyn ObjectStore>, location: Path) -> VortexResult<Self> {
        let upload = object_store.put_multipart(&location).await?;
        Ok(Self { upload })
    }
}

impl VortexWrite for ObjectStoreWriter {
    async fn write_all<B: IoBuf>(&mut self, buffer: B) -> io::Result<B> {
        const CHUNKS_SIZE: usize = 25 * 1024 * 1024;

        for chunk in buffer.as_slice().chunks(CHUNKS_SIZE) {
            let payload = Bytes::copy_from_slice(chunk);
            self.upload
                .as_mut()
                .put_part(PutPayload::from_bytes(payload))
                .await?;
        }

        Ok(buffer)
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.upload.complete().await?;
        Ok(())
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        Ok(())
    }
}
