pub mod base;
pub mod data;
pub mod helpers;
pub mod json;

use base::{PipelineJobContext, PipelineJobResult, RunNodeError};
use copper_storaged::{client::StoragedClient, Transaction, UserId};
use copper_util::s3client::S3Client;
use data::PipeData;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct JobRunResult {
	pub transaction: Transaction,
}

impl PipelineJobResult for JobRunResult {}

impl PipelineJobContext<PipeData, JobRunResult> for CopperContext {
	fn to_result(self) -> Result<JobRunResult, RunNodeError<PipeData>> {
		let transaction = self.transaction.into_inner();
		return Ok(JobRunResult { transaction });
	}
}
