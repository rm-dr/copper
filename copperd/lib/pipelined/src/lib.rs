pub mod base;
pub mod data;
pub mod helpers;

use base::PipelineJobContext;
use copper_storaged::client::StoragedClient;
use smartstring::{LazyCompact, SmartString};
use std::sync::Arc;

pub struct CopperContext {
	/// The maximum size, in bytes, of a blob channel fragment
	pub blob_fragment_size: u64,

	pub storaged_client: Arc<dyn StoragedClient>,
	pub objectstore_client: Arc<aws_sdk_s3::Client>,
	pub objectstore_bucket: String,
	pub job_id: SmartString<LazyCompact>,
}

impl PipelineJobContext for CopperContext {}
