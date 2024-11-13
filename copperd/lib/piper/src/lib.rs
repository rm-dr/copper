pub mod base;
pub mod data;
pub mod helpers;
pub mod json;

use copper_itemdb::{client::ItemdbClient, UserId};
use copper_util::s3client::S3Client;
use smartstring::{LazyCompact, SmartString};
use sqlx::{Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CopperContext<'a> {
	/// The fragment size, in bytes, in which we should read large blobs.
	///
	/// A larger value uses more memory, but increases performance
	/// (with diminishing returns)
	pub stream_fragment_size: usize,
	pub stream_channel_size: usize,

	/// The id of this job
	pub job_id: SmartString<LazyCompact>,

	/// The user running this pipeline.
	/// Used to make sure we have permission to do the
	/// actions in this pipeline.
	pub run_by_user: UserId,

	/// The itemdb client this runner should use
	pub itemdb_client: Arc<ItemdbClient>,

	/// The objectstore client this pipeline should use
	pub objectstore_client: Arc<S3Client>,

	/// The name of the bucket to store blobs in
	pub objectstore_blob_bucket: SmartString<LazyCompact>,

	/// The transaction we'll use to query the item db
	///
	/// This makes sure pipelines are atomic.
	/// We `.commit()` only if the pipeline runs successfully.
	pub item_db_transaction: Mutex<Transaction<'a, Postgres>>,
}
