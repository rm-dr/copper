use copper_util::MimeType;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
	base::RunNodeError,
	data::{BytesSource, FragmentArray, PipeData},
	CopperContext,
};

use super::s3client::S3Reader;

pub enum BytesSourceReader<'a> {
	Array {
		data: Option<Arc<Vec<u8>>>,
		mime: MimeType,
	},

	Stream {
		data: Arc<Mutex<FragmentArray>>,
		mime: MimeType,
	},
	S3(S3Reader<'a>),
}

impl<'a> BytesSourceReader<'a> {
	pub async fn open(
		ctx: &'a CopperContext,
		source: BytesSource,
	) -> Result<Self, RunNodeError<PipeData>> {
		return Ok(match source {
			BytesSource::Array { data, mime } => Self::Array {
				data: Some(data),
				mime,
			},

			BytesSource::Stream { fragments, mime } => Self::Stream {
				data: fragments,
				mime,
			},

			BytesSource::S3 { key } => Self::S3(
				ctx.objectstore_client
					.create_reader(&key)
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

			Self::Stream { .. } => {
				todo!()
			}

			Self::S3(reader) => {
				let mut read_buf = vec![0u8; max_buffer_size];

				let l = reader
					.read(&mut read_buf)
					.await
					.map_err(|e| RunNodeError::Other(Arc::new(e)))?;

				if l == 0 {
					return Ok(None);
				} else {
					read_buf.truncate(l);
					return Ok(Some(Arc::new(read_buf)));
				}
			}
		}
	}
}
