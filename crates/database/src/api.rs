use std::sync::Arc;

use ufo_db_blobstore::api::Blobstore;
use ufo_db_metastore::api::Metastore;
use ufo_db_pipestore::api::Pipestore;

pub trait UFODatabase
where
	Self: Send + Sync,
{
	fn get_metastore(&self) -> Arc<dyn Metastore>;
	fn get_pipestore(&self) -> Arc<dyn Pipestore>;
	fn get_blobstore(&self) -> Arc<dyn Blobstore>;
}
