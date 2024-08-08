use std::sync::Arc;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
};

pub trait Pipestore<PipelineNodeStubType: PipelineNodeStub>
where
	Self: Send + Sync,
{
	fn load_pipeline(
		&self,
		name: &PipelineName,
		context: Arc<<PipelineNodeStubType::NodeType as PipelineNode>::NodeContext>,
	) -> Option<Pipeline<PipelineNodeStubType>>;

	fn all_pipelines(&self) -> &Vec<PipelineName>;
}
