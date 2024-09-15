use std::{
	error::Error,
	fmt::Display,
	io::{Seek, SeekFrom, Write},
	sync::Arc,
};

use aws_sdk_s3::{
	error::SdkError,
	primitives::{ByteStream, SdkBody},
	types::{CompletedMultipartUpload, CompletedPart},
};
use copper_util::MimeType;
use smartstring::{LazyCompact, SmartString};

//
// MARK: Errors
//

#[derive(Debug)]
pub enum S3ReaderError {
	SdkError(Box<dyn std::error::Error + Send + Sync>),
	ByteStreamError(aws_sdk_s3::primitives::ByteStreamError),
}

impl<E: std::error::Error + 'static + Send + Sync, R: std::fmt::Debug + 'static + Send + Sync>
	From<SdkError<E, R>> for S3ReaderError
{
	fn from(value: SdkError<E, R>) -> Self {
		Self::SdkError(Box::new(value))
	}
}

impl From<aws_sdk_s3::primitives::ByteStreamError> for S3ReaderError {
	fn from(value: aws_sdk_s3::primitives::ByteStreamError) -> Self {
		Self::ByteStreamError(value)
	}
}

impl Display for S3ReaderError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::SdkError(_) => write!(f, "sdk error"),
			Self::ByteStreamError(_) => write!(f, "byte stream error"),
		}
	}
}

impl Error for S3ReaderError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::SdkError(x) => Some(&**x),
			Self::ByteStreamError(x) => Some(x),
		}
	}
}

#[derive(Debug)]
pub enum S3MultupartUploadError {
	SdkError(Box<dyn std::error::Error + Send + Sync>),
}

impl<E: std::error::Error + 'static + Send + Sync, R: std::fmt::Debug + 'static + Send + Sync>
	From<SdkError<E, R>> for S3MultupartUploadError
{
	fn from(value: SdkError<E, R>) -> Self {
		Self::SdkError(Box::new(value))
	}
}

impl Display for S3MultupartUploadError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::SdkError(_) => write!(f, "sdk error"),
		}
	}
}

impl Error for S3MultupartUploadError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::SdkError(x) => Some(&**x),
		}
	}
}

//
// MARK: Implementations
//

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
	pub async fn create_reader(&'a self, key: &str) -> Result<S3Reader<'a>, S3ReaderError> {
		let b = self
			.client
			.get_object()
			.bucket(&self.bucket)
			.key(key)
			.send()
			.await?;

		return Ok(S3Reader {
			client: self,

			key: key.into(),
			cursor: 0,
			// TODO: when does this fail?
			size: b.content_length.unwrap().try_into().unwrap(),
			mime: b.content_type.map(MimeType::from).unwrap_or(MimeType::Blob),
		});
	}

	pub async fn create_multipart_upload(
		&'a self,
		key: &str,
		mime: MimeType,
	) -> Result<MultipartUpload<'a>, S3MultupartUploadError> {
		let multipart_upload_res = self
			.client
			.create_multipart_upload()
			.bucket(&self.bucket)
			.key(key)
			.content_type(&mime)
			.send()
			.await?;

		let upload_id = multipart_upload_res.upload_id().unwrap();

		return Ok(MultipartUpload {
			client: self,
			key: key.into(),
			id: upload_id.into(),
			completed_parts: Vec::new(),
		});
	}
}

pub struct S3Reader<'a> {
	client: &'a S3Client,

	key: SmartString<LazyCompact>,
	cursor: u64,
	size: u64,
	mime: MimeType,
}

impl S3Reader<'_> {
	pub async fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, S3ReaderError> {
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
			.await?;

		// Looks like `bytes 31000000-31999999/33921176``
		// println!("{:?}", b.content_range);

		let mut bytes = b.body.collect().await?.into_bytes();
		bytes.truncate(len_to_read);
		let l = bytes.len();

		// Memory to memory writes should not fail
		buf.write_all(&bytes).unwrap();

		self.cursor += u64::try_from(l).unwrap();
		return Ok(len_to_read);
	}

	pub fn is_done(&self) -> bool {
		return self.cursor == self.size;
	}

	pub fn mime(&self) -> &MimeType {
		&self.mime
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
	pub async fn upload_part(
		&mut self,
		data: &[u8],
		part_number: i32,
	) -> Result<(), S3MultupartUploadError> {
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
			.await?;

		self.completed_parts.push(
			CompletedPart::builder()
				.e_tag(upload_part_res.e_tag.unwrap_or_default())
				.part_number(part_number)
				.build(),
		);

		return Ok(());
	}

	pub async fn finish(self) -> Result<(), S3MultupartUploadError> {
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
			.await?;

		return Ok(());
	}
}
