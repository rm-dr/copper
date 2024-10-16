pub mod base;
pub mod client;
pub mod data;
pub mod helpers;
pub mod json;
pub mod structs;

use async_trait::async_trait;
use base::{PipelineJobContext, RunNodeError};
use copper_storaged::{client::StoragedClient, Transaction, UserId};
use copper_util::s3client::S3Client;
use data::PipeData;
use smartstring::{LazyCompact, SmartString};
use std::sync::Arc;
use tokio::sync::Mutex;

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

	/// The id of this job
	pub job_id: SmartString<LazyCompact>,

	/// The user running this pipeline.
	/// Used to make sure we have permission to do the
	/// actions in this pipeline.
	pub run_by_user: UserId,

	/// The storaged client this pipeline should use
	pub storaged_client: Arc<dyn StoragedClient>,

	/// The objectstore client this pipeline should use
	pub objectstore_client: Arc<S3Client>,

	/// The name of the bucket to store blobs in
	pub objectstore_blob_bucket: SmartString<LazyCompact>,

	/// The transaction to apply once this pipeline successfully resolves.
	/// A pipeline triggers AT MOST one transaction.
	pub transaction: Mutex<Transaction>,
}

#[async_trait]
impl PipelineJobContext<PipeData> for CopperContext {
	async fn on_complete(self) -> Result<(), RunNodeError<PipeData>> {
		let transaction = self.transaction.into_inner();
		if !transaction.is_empty() {
			self.storaged_client
				.apply_transaction(transaction)
				.await
				.map_err(|e| RunNodeError::Other(Arc::new(e)))?;
		}

		return Ok(());
	}
}
