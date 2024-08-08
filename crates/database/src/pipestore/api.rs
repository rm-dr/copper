use ufo_pipeline::labels::PipelineLabel;

pub trait Pipestore
where
	Self: Send,
{
	fn load_pipeline(&self, name: PipelineLabel) -> String;
	fn all_pipelines(&self) -> &[String];
}

/*
pub trait Pipestore<NodeStub: PipelineNodeStub>
where
	Self: Send + Sized,
{
	fn load_pipeline(
		&self,
		name: PipelineLabel,
		context: Arc<<NodeStub::NodeType as PipelineNode>::NodeContext>,
	) -> Pipeline<NodeStub>;
}
*/
