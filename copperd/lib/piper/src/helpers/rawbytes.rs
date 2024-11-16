use copper_util::{s3client::S3Reader, MimeType};
use smartstring::{LazyCompact, SmartString};
use std::sync::Arc;

use crate::{base::RunNodeError, CopperContext};

/// An unprocessed source of binary data
#[derive(Debug, Clone)]
pub enum RawBytesSource {
	Array {
		mime: MimeType,
		data: Arc<Vec<u8>>,
	},

	S3 {
		key: SmartString<LazyCompact>,
		bucket: SmartString<LazyCompact>,
	},
}

pub(crate) enum RawBytesSourceReader {
	Array {
		data: Option<Arc<Vec<u8>>>,
		mime: MimeType,
	},

	S3(S3Reader),
}

impl RawBytesSourceReader {
	pub async fn open(
		ctx: &CopperContext<'_>,
		source: RawBytesSource,
	) -> Result<Self, RunNodeError> {
		return Ok(match source {
			RawBytesSource::Array { data, mime } => Self::Array {
				data: Some(data),
				mime,
			},

			RawBytesSource::S3 { bucket, key } => Self::S3(
				ctx.objectstore_client
					.create_reader(&bucket, &key)
					.await
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?,
			),
		});
	}

	pub fn mime(&self) -> &MimeType {
		match self {
			Self::Array { mime, .. } => mime,
			Self::S3(reader) => reader.mime(),
		}
	}

	/// Read the next fragment from this bytes source.
	/// If there is no more data to read, return `None`.
	pub async fn next_fragment(
		&mut self,
		max_buffer_size: usize,
	) -> Result<Option<Arc<Vec<u8>>>, RunNodeError> {
		match self {
			Self::Array { data, .. } => return Ok(data.take()),

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
