use copper_ds_core::{api::pipe::Pipestore, errors::PipestoreError};
use copper_pipeline::{
	api::{PipelineData, PipelineJobContext},
	dispatcher::NodeDispatcher,
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
};
use sqlx::Row;

use super::LocalDataset;

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	Pipestore<DataType, ContextType> for LocalDataset
{
	async fn load_pipeline(
		&self,
		dispatcher: &NodeDispatcher<DataType, ContextType>,
		context: &ContextType,
		name: &PipelineName,
	) -> Result<Option<Pipeline<DataType, ContextType>>, PipestoreError<DataType>> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| PipestoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT pipeline_data FROM meta_pipelines WHERE pipeline_name=?;")
			.bind(name.to_string())
			.fetch_one(&mut *conn)
			.await;

		// IMPORTANT!
		// from_toml_str also locks this connection,
		// and will deadlock if we don't drop here
		drop(conn);

		let pipe_spec = match res {
			Err(sqlx::Error::RowNotFound) => return Ok(None),
			Err(e) => return Err(PipestoreError::DbError(Box::new(e))),
			Ok(res) => res.get::<String, _>("pipeline_data"),
		};

		return Ok(Some(Pipeline::from_toml_str(
			dispatcher, context, name, &pipe_spec,
		)?));
	}

	async fn all_pipelines(&self) -> Result<Vec<PipelineName>, PipestoreError<DataType>> {
		let mut conn = self
			.pool
			.acquire()
			.await
			.map_err(|e| PipestoreError::DbError(Box::new(e)))?;

		let res = sqlx::query("SELECT pipeline_name FROM meta_pipelines ORDER BY id;")
			.fetch_all(&mut *conn)
			.await
			.map_err(|e| PipestoreError::DbError(Box::new(e)))?;

		return Ok(res
			.into_iter()
			.map(|x| x.get::<String, _>("pipeline_name"))
			.map(|x| PipelineName::new(&x))
			.collect());
	}
}
