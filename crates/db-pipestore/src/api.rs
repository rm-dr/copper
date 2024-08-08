use std::sync::Arc;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
};
use ufo_pipeline_nodes::nodetype::UFONodeType;

pub trait Pipestore
where
	Self: Send + Sync,
{
	fn load_pipeline(
		&self,
		name: &PipelineName,
		context: Arc<<<UFONodeType as PipelineNodeStub>::NodeType as PipelineNode>::NodeContext>,
	) -> Option<Pipeline<UFONodeType>>;

	fn all_pipelines(&self) -> &Vec<PipelineName>;
}
