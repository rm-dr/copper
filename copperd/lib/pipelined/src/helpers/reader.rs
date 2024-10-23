use copper_storage::database::base::client::StorageDatabaseClient;
use copper_util::{s3client::S3Reader, MimeType};
use std::sync::Arc;

use crate::{
	base::RunNodeError,
	data::{BytesSource, PipeData},
	CopperContext,
};

pub enum BytesSourceReader {
	Array {
		data: Option<Arc<Vec<u8>>>,
		mime: MimeType,
	},

	Stream {
		receiver: async_broadcast::Receiver<Arc<Vec<u8>>>,
		mime: MimeType,
	},
	S3(S3Reader),
}

impl BytesSourceReader {
	pub async fn open<StorageClient: StorageDatabaseClient>(
		ctx: &CopperContext<StorageClient>,
		source: BytesSource,
	) -> Result<Self, RunNodeError<PipeData>> {
		return Ok(match source {
			BytesSource::Array { data, mime } => Self::Array {
				data: Some(data),
				mime,
			},

			BytesSource::Stream { receiver, mime } => Self::Stream { receiver, mime },

			BytesSource::S3 { bucket, key } => Self::S3(
				ctx.objectstore_client
					.create_reader(&bucket, &key)
					.await
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?,
			),
		});
	}

	pub fn mime(&self) -> &MimeType {
		match self {
			Self::Stream { mime, .. } | Self::Array { mime, .. } => mime,
			Self::S3(reader) => reader.mime(),
		}
	}

	/// Read the next fragment from this bytes source.
	/// If there is no more data to read, return `None`.
	pub async fn next_fragment(
		&mut self,
		max_buffer_size: usize,
	) -> Result<Option<Arc<Vec<u8>>>, RunNodeError<PipeData>> {
		match self {
			Self::Array { data, .. } => return Ok(data.take()),

			Self::Stream { receiver, .. } => {
				match receiver.recv().await {
					Ok(x) => return Ok(Some(x)),
					Err(async_broadcast::RecvError::Closed) => return Ok(None),
					Err(async_broadcast::RecvError::Overflowed(_)) => {
						return Err(RunNodeError::StreamReceiverOverflowed)
					}
				};
			}

			Self::S3(reader) => {
				if reader.is_done() {
					return Ok(None);
				}

				let mut read_buf = vec![0u8; max_buffer_size];
				let l = reader
					.read(&mut read_buf)
					.await
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

				read_buf.truncate(l);
				return Ok(Some(Arc::new(read_buf)));
			}
		}
	}
}
