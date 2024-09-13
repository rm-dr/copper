pub mod base;
pub mod data;
pub mod helpers;

use base::PipelineJobContext;
use copper_storaged::client::StoragedClient;
use smartstring::{LazyCompact, SmartString};
use std::sync::Arc;

pub struct CopperContext {
	/// The fragment size, in bytes, in which we should read large blobs.
	///
	/// A larger value uses more memory, but increases performance
	/// (with diminishing returns)
	pub blob_fragment_size: usize,

	/// The message capacity of binary stream channels.
	///
	/// Smaller values increase the probability of pipeline runs failing due to an
	/// overflowing channel, larger values use more memory.
	pub stream_channel_capacity: usize,

	pub storaged_client: Arc<dyn StoragedClient>,
	pub objectstore_client: Arc<aws_sdk_s3::Client>,
	pub objectstore_bucket: String,
	pub job_id: SmartString<LazyCompact>,
}

impl PipelineJobContext for CopperContext {}
