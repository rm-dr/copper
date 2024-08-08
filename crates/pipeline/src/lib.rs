pub mod errors;
pub mod input;
pub mod nodes;
pub mod output;
#[allow(clippy::module_inception)]
pub mod pipeline;
pub mod syntax;

use errors::PipelineError;
use std::sync::Arc;
use ufo_util::data::PipelineData;

pub trait PipelineStatelessRunner {
	fn run(
		&self,
		input: Vec<Option<Arc<PipelineData>>>,
	) -> Result<Vec<Option<Arc<PipelineData>>>, PipelineError>;
}
