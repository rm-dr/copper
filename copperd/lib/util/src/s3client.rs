use aws_sdk_s3::{
	error::SdkError,
	primitives::{ByteStream, SdkBody},
	types::{CompletedMultipartUpload, CompletedPart},
};
use smartstring::{LazyCompact, SmartString};
use std::{
	fmt::Debug,
	io::{Seek, SeekFrom, Write},
};
use thiserror::Error;
use tracing::error;

use crate::MimeType;

//
// MARK: Errors
//

#[derive(Debug, Error)]
pub enum S3ReaderError {
	#[error("sdk error")]
	SdkError(#[from] Box<dyn std::error::Error + Send + Sync>),

	#[error("byte stream error")]
	ByteStreamError(#[from] aws_sdk_s3::primitives::ByteStreamError),
}

impl<E: std::error::Error + 'static + Send + Sync, R: std::fmt::Debug + 'static + Send + Sync>
	From<SdkError<E, R>> for S3ReaderError
{
	fn from(value: SdkError<E, R>) -> Self {
		Self::SdkError(Box::new(value))
	}
}

#[derive(Debug, Error)]
pub enum S3UploadPartError {
	#[error("sdk error")]
	SdkError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl<E: std::error::Error + 'static + Send + Sync, R: std::fmt::Debug + 'static + Send + Sync>
	From<SdkError<E, R>> for S3UploadPartError
{
	fn from(value: SdkError<E, R>) -> Self {
		Self::SdkError(Box::new(value))
	}
}

#[derive(Debug, Error)]
pub enum S3CreateMultipartUploadError {
	#[error("sdk error")]
	SdkError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl<E: std::error::Error + 'static + Send + Sync, R: std::fmt::Debug + 'static + Send + Sync>
	From<SdkError<E, R>> for S3CreateMultipartUploadError
{
	fn from(value: SdkError<E, R>) -> Self {
		Self::SdkError(Box::new(value))
	}
}

#[derive(Debug, Error)]
pub enum S3UploadFinishError {
	#[error("sdk error")]
	SdkError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl<E: std::error::Error + 'static + Send + Sync, R: std::fmt::Debug + 'static + Send + Sync>
	From<SdkError<E, R>> for S3UploadFinishError
{
	fn from(value: SdkError<E, R>) -> Self {
		Self::SdkError(Box::new(value))
	}
}

#[derive(Debug, Error)]
pub enum S3DeleteObjectError {
	#[error("sdk error")]
	SdkError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl<E: std::error::Error + 'static + Send + Sync, R: std::fmt::Debug + 'static + Send + Sync>
	From<SdkError<E, R>> for S3DeleteObjectError
{
	fn from(value: SdkError<E, R>) -> Self {
		Self::SdkError(Box::new(value))
	}
}

#[derive(Debug, Error)]
pub enum S3CreateBucketError {
	#[error("sdk error")]
	SdkError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl<E: std::error::Error + 'static + Send + Sync, R: std::fmt::Debug + 'static + Send + Sync>
	From<SdkError<E, R>> for S3CreateBucketError
{
	fn from(value: SdkError<E, R>) -> Self {
		Self::SdkError(Box::new(value))
	}
}

//
// MARK: Implementations
//

pub struct ObjectMetadata {
	pub mime: MimeType,
	pub size: Option<i64>,
}

/// An interface to a specific S3 bucket
#[derive(Clone)]
pub struct S3Client {
	client: aws_sdk_s3::Client,
}

impl S3Client {
	pub async fn new(client: aws_sdk_s3::Client) -> Self {
		Self { client }
	}
}

impl<'a> S3Client {
	pub async fn get_object_stream(
		&'a self,
		bucket: &str,
		key: &str,
	) -> Result<ByteStream, S3ReaderError> {
		let b = self
			.client
			.get_object()
			.bucket(bucket)
			.key(key)
			.send()
			.await?;

		return Ok(b.body);
	}

	pub async fn get_object_metadata(
		&'a self,
		bucket: &str,
		key: &str,
	) -> Result<ObjectMetadata, S3ReaderError> {
		let b = self
			.client
			.get_object()
			.bucket(bucket)
			.key(key)
			.send()
			.await?;

		let mime = b
			.content_type()
			.map(MimeType::from)
			.unwrap_or(MimeType::Other("application/octet-stream".into()));

		let size = b.content_length();

		return Ok(ObjectMetadata { mime, size });
	}

	pub async fn create_reader(
		&'a self,
		bucket: &str,
		key: &str,
	) -> Result<S3Reader, S3ReaderError> {
		let b = self
			.client
			.get_object()
			.bucket(bucket)
			.key(key)
			.send()
			.await?;

		return Ok(S3Reader {
			client: self.clone(),
			bucket: bucket.into(),
			key: key.into(),

			cursor: 0,
			// TODO: when does this fail?
			size: b.content_length.unwrap().try_into().unwrap(),
			mime: b.content_type.map(MimeType::from).unwrap_or(MimeType::Blob),
		});
	}

	pub async fn create_multipart_upload(
		&'a self,
		bucket: &str,
		key: &str,
		mime: MimeType,
	) -> Result<MultipartUpload, S3CreateMultipartUploadError> {
		let multipart_upload_res = self
			.client
			.create_multipart_upload()
			.bucket(bucket)
			.key(key)
			.content_type(&mime)
			.send()
			.await?;

		let upload_id = multipart_upload_res.upload_id().unwrap();

		return Ok(MultipartUpload {
			client: self.clone(),
			bucket: bucket.into(),
			key: key.into(),

			id: upload_id.into(),
			completed_parts: Vec::new(),
		});
	}

	pub async fn delete_object(
		&'a self,
		bucket: &str,
		key: &str,
	) -> Result<(), S3DeleteObjectError> {
		self.client
			.delete_object()
			.bucket(bucket)
			.key(key)
			.send()
			.await?;

		return Ok(());
	}

	/// Create a bucket if it doesn't exist.
	/// Returns `true` if a bucket was created, and `false` if it was already there.
	pub async fn create_bucket(&'a self, bucket: &str) -> Result<bool, S3CreateBucketError> {
		let res = self.client.create_bucket().bucket(bucket).send().await;

		if let Err(err) = res {
			match err.as_service_error() {
				None => return Err(err.into()),
				Some(x) => {
					if x.is_bucket_already_exists() || x.is_bucket_already_owned_by_you() {
						return Ok(false);
					} else {
						return Err(err.into());
					}
				}
			}
		} else {
			return Ok(true);
		}
	}
}

//
// MARK: S3Reader
//

pub struct S3Reader {
	client: S3Client,
	bucket: SmartString<LazyCompact>,
	key: SmartString<LazyCompact>,

	cursor: u64,
	size: u64,
	mime: MimeType,
}

impl S3Reader {
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
			.bucket(self.bucket.as_str())
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

pub struct MultipartUpload {
	client: S3Client,
	bucket: SmartString<LazyCompact>,
	key: SmartString<LazyCompact>,

	id: SmartString<LazyCompact>,
	completed_parts: Vec<CompletedPart>,
}

impl MultipartUpload {
	pub fn n_completed_parts(&self) -> usize {
		self.completed_parts.len()
	}

	pub fn key(&self) -> &str {
		&self.key
	}

	/// Upload a part to a multipart upload.
	/// `part_number` must be consecutive, and starts at 1.
	pub async fn upload_part(
		&mut self,
		data: &[u8],
		part_number: i32,
	) -> Result<(), S3UploadPartError> {
		let stream = ByteStream::from(SdkBody::from(data));

		// Chunk index needs to start at 0, but part numbers start at 1.
		let upload_part_res = self
			.client
			.client
			.upload_part()
			.bucket(self.bucket.as_str())
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

	/// Cancel this multipart upload.
	/// This catches and logs all errors.
	pub async fn cancel(self) {
		let res = self
			.client
			.client
			.abort_multipart_upload()
			.bucket(self.bucket.as_str())
			.key(self.key.as_str())
			.upload_id(self.id.clone())
			.send()
			.await;

		if let Err(error) = res {
			error!(message = "Error while canceling job", ?error);
		}
	}

	pub async fn finish(self) -> Result<(), S3UploadFinishError> {
		let completed_multipart_upload = CompletedMultipartUpload::builder()
			.set_parts(Some(self.completed_parts))
			.build();

		self.client
			.client
			.complete_multipart_upload()
			.bucket(self.bucket.as_str())
			.key(self.key.as_str())
			.upload_id(self.id.clone())
			.multipart_upload(completed_multipart_upload)
			.send()
			.await?;

		return Ok(());
	}
}

impl Debug for MultipartUpload {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"MultipartUpload{{bucket: {}, key: {}}}",
			self.bucket, self.key
		)
	}
}
