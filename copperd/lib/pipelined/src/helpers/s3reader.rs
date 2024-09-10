use std::{
	io::{Read, Seek, SeekFrom, Write},
	sync::Arc,
};

use copper_util::MimeType;
use futures::executor::block_on;
use tracing::error;

pub struct S3Reader {
	client: Arc<aws_sdk_s3::Client>,
	bucket: String,
	key: String,

	cursor: u64,
	size: u64,
	pub mime: MimeType,
}

/// Provides an *unbuffered* `Read + Seek` interface to an S3 object.
/// Each call to `read()` results in a blocking http `range` request.
///
/// It may be wise to attach a buffer to an [`S3Reader`].
impl S3Reader {
	pub async fn new(
		client: Arc<aws_sdk_s3::Client>,
		bucket: impl ToString,
		key: impl ToString,
	) -> Self {
		let bucket = bucket.to_string();
		let key = key.to_string();

		let b = client
			.get_object()
			.bucket(&bucket)
			.key(&key)
			.send()
			.await
			.unwrap();

		Self {
			client,
			bucket,
			key,

			cursor: 0,
			size: b.content_length.unwrap().try_into().unwrap(),
			mime: b
				.content_type
				.map(|x| MimeType::try_from(x).unwrap())
				.unwrap_or(MimeType::Blob),
		}
	}

	pub fn is_done(&self) -> bool {
		error!(c = self.cursor, s = self.size);
		return self.cursor == self.size - 1;
	}
}

impl Read for S3Reader {
	fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
		let len_left = usize::try_from(self.size - self.cursor).unwrap();
		if len_left == 0 || buf.len() == 0 {
			return Ok(0);
		}

		let start_byte = usize::try_from(self.cursor).unwrap();
		let len_to_read = buf.len().min(len_left);
		let end_byte = start_byte + len_to_read - 1;

		let b = block_on(
			self.client
				.get_object()
				.bucket(&self.bucket)
				.key(&self.key)
				.range(format!("bytes={start_byte}-{end_byte}"))
				.send(),
		)
		.unwrap();

		// Looks like `bytes 31000000-31999999/33921176``
		// println!("{:?}", b.content_range);

		let mut bytes = block_on(b.body.collect())?.into_bytes();
		bytes.truncate(len_to_read);
		let l = bytes.len();
		buf.write_all(&bytes)?;

		self.cursor += u64::try_from(l).unwrap();
		return Ok(len_to_read);
	}
}

impl Seek for S3Reader {
	fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
		match pos {
			SeekFrom::Start(x) => self.cursor = x.min(self.size - 1),

			SeekFrom::Current(x) => {
				if x < 0 {
					if u64::try_from(x.abs()).unwrap() > self.cursor {
						return Err(std::io::Error::new(
							std::io::ErrorKind::InvalidInput,
							"cannot seek past start",
						));
					}
					self.cursor -= u64::try_from(x.abs()).unwrap();
				} else {
					self.cursor += u64::try_from(x).unwrap();
				}
			}

			SeekFrom::End(x) => {
				if x < 0 {
					if u64::try_from(x.abs()).unwrap() > self.size {
						return Err(std::io::Error::new(
							std::io::ErrorKind::InvalidInput,
							"cannot seek past start",
						));
					}
					self.cursor = self.size - u64::try_from(x.abs()).unwrap();
				} else {
					self.cursor = self.size + u64::try_from(x).unwrap();
				}
			}
		}

		self.cursor = self.cursor.min(self.size - 1);
		return Ok(self.cursor);
	}
}
