use std::{collections::HashMap, sync::Arc};

use crate::pipeline::{data::PipelineData, syntax::labels::PipelinePortLabel};

pub mod file;

pub trait Ingest {
	type ErrorKind: Send + Sync;

	fn injest(
		self,
	) -> Result<HashMap<PipelinePortLabel, Option<Arc<PipelineData>>>, Self::ErrorKind>;
}
