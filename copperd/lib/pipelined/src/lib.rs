pub mod base;
pub mod data;
pub mod helpers;
pub mod json;

use async_trait::async_trait;
use base::{PipelineJobContext, RunNodeError};
use copper_storaged::{client::StoragedClient, Transaction};
use data::PipeData;
use helpers::S3Client;
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

	pub storaged_client: Arc<dyn StoragedClient>,
	pub objectstore_client: Arc<S3Client>,
	pub job_id: SmartString<LazyCompact>,

	/// The transaction to apply once this pipeline successfully resolves.
	/// A pipeline should trigger AT MOST one transaction.
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
