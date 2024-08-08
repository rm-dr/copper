use std::sync::Arc;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
};

use crate::errors::PipestoreError;

#[allow(async_fn_in_trait)]
pub trait Pipestore<PipelineNodeStubType: PipelineNodeStub>
where
	Self: Send + Sync,
{
	async fn load_pipeline(
		&self,
		name: &PipelineName,
		context: Arc<<PipelineNodeStubType::NodeType as PipelineNode>::NodeContext>,
	) -> Result<Option<Pipeline<PipelineNodeStubType>>, PipestoreError<PipelineNodeStubType>>;

	// TODO: cache list of pipelines?
	async fn all_pipelines(
		&self,
	) -> Result<Vec<PipelineName>, PipestoreError<PipelineNodeStubType>>;
}
