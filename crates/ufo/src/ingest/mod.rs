use std::sync::Arc;
use ufo_pipeline::data::PipelineData;

pub mod file;

pub trait Ingest {
	type ErrorKind: Send + Sync;

	fn injest(self) -> Result<Vec<Option<Arc<PipelineData>>>, Self::ErrorKind>;
}
