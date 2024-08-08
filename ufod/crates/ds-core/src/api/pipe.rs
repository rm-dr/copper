use std::sync::Arc;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
};

use crate::errors::PipestoreError;

pub trait Pipestore<PipelineNodeStubType: PipelineNodeStub>
where
	Self: Send + Sync,
{
	fn load_pipeline(
		&self,
		name: &PipelineName,
		context: Arc<<PipelineNodeStubType::NodeType as PipelineNode>::NodeContext>,
	) -> Result<Option<Pipeline<PipelineNodeStubType>>, PipestoreError>;

	// TODO: cache list of pipelines?
	fn all_pipelines(&self) -> Result<Vec<PipelineName>, PipestoreError>;
}
