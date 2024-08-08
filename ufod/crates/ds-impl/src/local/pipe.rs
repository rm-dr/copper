use futures::executor::block_on;
use sqlx::Row;
use std::sync::Arc;
use ufo_ds_core::{api::pipe::Pipestore, errors::PipestoreError};
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
	) -> Result<Option<Pipeline<PipelineNodeStubType>>, PipestoreError<PipelineNodeStubType>> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query("SELECT pipeline_data FROM meta_pipelines WHERE pipeline_name=?;")
				.bind(name.to_string())
				.fetch_one(&mut *conn),
		);

		// IMPORTANT!
		// from_toml_str also locks this connection,
		// and will deadlock if we don't drop here
		drop(conn);

		let pipe_spec = match res {
			Err(sqlx::Error::RowNotFound) => return Ok(None),
			Err(e) => return Err(PipestoreError::DbError(Box::new(e))),
			Ok(res) => res.get::<String, _>("pipeline_data"),
		};

		return Ok(Some(Pipeline::<PipelineNodeStubType>::from_toml_str(
			name, &pipe_spec, context,
		)?));
	}

	fn all_pipelines(&self) -> Result<Vec<PipelineName>, PipestoreError<PipelineNodeStubType>> {
		let mut conn = self.conn.lock().unwrap();

		let res = block_on(
			sqlx::query("SELECT pipeline_name FROM meta_pipelines ORDER BY id;")
				.fetch_all(&mut *conn),
		)
		.map_err(|e| PipestoreError::DbError(Box::new(e)))?;

		return Ok(res
			.into_iter()
			.map(|x| x.get::<String, _>("pipeline_name"))
			.map(|x| PipelineName::new(&x))
			.collect());
	}
}
