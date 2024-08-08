use std::sync::Arc;
use ufo_ds_core::api::pipe::Pipestore;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
};

use super::LocalDataset;

impl<PipelineNodeStubType: PipelineNodeStub> Pipestore<PipelineNodeStubType> for LocalDataset {
	fn load_pipeline(
		&self,
		name: &PipelineName,
		context: Arc<<PipelineNodeStubType::NodeType as PipelineNode>::NodeContext>,
	) -> Option<Pipeline<PipelineNodeStubType>> {
		todo!()
	}

	fn all_pipelines(&self) -> &Vec<PipelineName> {
		todo!()
	}
}
