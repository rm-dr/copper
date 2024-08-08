// Utilities
pub mod nodeinstance;
pub mod nodetype;

// Node implementations
pub mod tags;
pub mod util;

use ufo_util::data::PipelineData;

use crate::errors::PipelineError;

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
