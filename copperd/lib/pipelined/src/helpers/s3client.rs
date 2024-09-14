use std::{
	io::{Seek, SeekFrom, Write},
	sync::Arc,
};

use aws_sdk_s3::{
	primitives::{ByteStream, SdkBody},
	types::{CompletedMultipartUpload, CompletedPart},
};
use copper_util::MimeType;
use smartstring::{LazyCompact, SmartString};

pub struct S3Client {
	client: Arc<aws_sdk_s3::Client>,
	bucket: String,
}

/// Provides an unbuffered interface to an S3 object.
///
///
impl S3Client {
	pub async fn new(client: Arc<aws_sdk_s3::Client>, bucket: impl ToString) -> Self {
		let bucket = bucket.to_string();
		Self { client, bucket }
	}
}

impl<'a> S3Client {
	pub async fn create_reader(&'a self, key: &str) -> S3Reader<'a> {
		let b = self
			.client
			.get_object()
			.bucket(&self.bucket)
			.key(key)
			.send()
			.await
			.unwrap();

		return S3Reader {
			client: self,

			key: key.into(),
			cursor: 0,
			size: b.content_length.unwrap().try_into().unwrap(),
			mime: b.content_type.map(MimeType::from).unwrap_or(MimeType::Blob),
		};
	}

	pub async fn create_multipart_upload(&'a self, key: &str) -> MultipartUpload<'a> {
		let multipart_upload_res = self
			.client
			.create_multipart_upload()
			.bucket(&self.bucket)
			.key(key)
			.send()
			.await
			.unwrap();

		let upload_id = multipart_upload_res.upload_id().unwrap();

		return MultipartUpload {
			client: self,
			key: key.into(),
			id: upload_id.into(),
			completed_parts: Vec::new(),
		};
	}
}

pub struct S3Reader<'a> {
	client: &'a S3Client,

	key: SmartString<LazyCompact>,
	cursor: u64,
	size: u64,
	pub mime: MimeType,
}

impl S3Reader<'_> {
	pub async fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
		let len_left = usize::try_from(self.size - self.cursor).unwrap();
		if len_left == 0 || buf.is_empty() {
			return Ok(0);
		}

		let start_byte = usize::try_from(self.cursor).unwrap();
		let len_to_read = buf.len().min(len_left);
		let end_byte = start_byte + len_to_read - 1;

		let b = self
			.client
			.client
			.get_object()
			.bucket(&self.client.bucket)
			.key(self.key.as_str())
			.range(format!("bytes={start_byte}-{end_byte}"))
			.send()
			.await
			.unwrap();

		// Looks like `bytes 31000000-31999999/33921176``
		// println!("{:?}", b.content_range);

		let mut bytes = b.body.collect().await?.into_bytes();
		bytes.truncate(len_to_read);
		let l = bytes.len();
		buf.write_all(&bytes)?;

		self.cursor += u64::try_from(l).unwrap();
		return Ok(len_to_read);
	}

	pub fn is_done(&self) -> bool {
		return self.cursor == self.size;
	}
}

impl Seek for S3Reader<'_> {
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

pub struct MultipartUpload<'a> {
	client: &'a S3Client,

	key: SmartString<LazyCompact>,
	id: SmartString<LazyCompact>,
	completed_parts: Vec<CompletedPart>,
}

impl MultipartUpload<'_> {
	/// Upload a part to a multipart upload.
	/// `part_number` must be consecutive, and starts at 1.
	pub async fn upload_part(&mut self, data: &[u8], part_number: i32) {
		let stream = ByteStream::from(SdkBody::from(data));

		// Chunk index needs to start at 0, but part numbers start at 1.
		let upload_part_res = self
			.client
			.client
			.upload_part()
			.bucket(&self.client.bucket)
			.key(self.key.as_str())
			.upload_id(self.id.clone())
			.body(stream)
			.part_number(part_number)
			.send()
			.await
			.unwrap();

		self.completed_parts.push(
			CompletedPart::builder()
				.e_tag(upload_part_res.e_tag.unwrap_or_default())
				.part_number(part_number)
				.build(),
		);
	}

	pub async fn finish(self) {
		let completed_multipart_upload = CompletedMultipartUpload::builder()
			.set_parts(Some(self.completed_parts))
			.build();

		self.client
			.client
			.complete_multipart_upload()
			.bucket(&self.client.bucket)
			.key(self.key.as_str())
			.upload_id(self.id.clone())
			.multipart_upload(completed_multipart_upload)
			.send()
			.await
			.unwrap();
	}
}
