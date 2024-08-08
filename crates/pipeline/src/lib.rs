//#![warn(missing_docs)]

pub mod errors;
pub mod input;
pub mod nodes;
pub mod output;
#[allow(clippy::module_inception)]
pub mod pipeline;
pub mod portspec;
pub mod runner;
pub mod syntax;

use errors::PipelineError;
use ufo_util::data::PipelineData;

pub trait PipelineNode {
	fn run<F>(
		&self,
		// Call this when data is ready.
		// Arguments are (port idx, data).
		//
		// This must be called *exactly once* for each of this port's outputs.
		// (not enforced, but the pipeline will panic or hang if this is violated.)
		send_data: F,
		input: Vec<PipelineData>,
	) -> Result<(), PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>;
}
