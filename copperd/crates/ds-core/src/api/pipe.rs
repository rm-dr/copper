use copper_pipeline::{
	api::{PipelineData, PipelineJobContext},
	dispatcher::NodeDispatcher,
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
};

use crate::errors::PipestoreError;

#[allow(async_fn_in_trait)]
pub trait Pipestore<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
where
	Self: Send + Sync,
{
	async fn load_pipeline(
		&self,
		dispatcher: &NodeDispatcher<DataType, ContextType>,
		context: &ContextType,
		name: &PipelineName,
	) -> Result<Option<Pipeline<DataType, ContextType>>, PipestoreError<DataType>>;

	async fn all_pipelines(&self) -> Result<Vec<PipelineName>, PipestoreError<DataType>>;
}
