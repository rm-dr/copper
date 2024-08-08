pub mod data;
pub mod errors;
pub mod nodes;
#[allow(clippy::module_inception)]
pub mod pipeline;
pub mod syntax;

use std::sync::Arc;

use self::{data::PipelineData, errors::PipelineError};

pub trait PipelineStatelessRunner {
	fn run(
		&self,
		input: Vec<Option<Arc<PipelineData>>>,
	) -> Result<Vec<Option<Arc<PipelineData>>>, PipelineError>;
}
