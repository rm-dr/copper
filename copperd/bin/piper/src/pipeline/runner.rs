use copper_jobqueue::id::QueuedJobId;
use copper_piper::{
	base::{NodeDispatcher, RunNodeError},
	data::PipeData,
	json::PipelineJson,
	CopperContext,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;
use tracing::debug;

use super::job::PipelineBuildError;
use crate::pipeline::job::PipelineJob;

//
// MARK: Errors
//

#[derive(Debug, Error)]
pub enum StartJobError {
	#[error("pipeline build error")]
	BuildError(#[from] PipelineBuildError),
}

//
// MARK: Runner
//

pub struct PipelineRunner {
	dispatcher: NodeDispatcher,
}

impl PipelineRunner {
	/// Initialize a new runner
	pub fn new() -> Self {
		Self {
			dispatcher: NodeDispatcher::new(),
		}
	}

	/// Start a job in this runner
	pub async fn run_job(
		&mut self,
		context: CopperContext<'_>,
		pipeline: PipelineJson,
		job_id: &QueuedJobId,
		inputs: BTreeMap<SmartString<LazyCompact>, PipeData>,
	) -> Result<Result<(), RunNodeError>, StartJobError> {
		debug!(message = "Starting job", ?job_id,);

		let job = PipelineJob::new(&self.dispatcher, job_id.as_str(), inputs.clone(), &pipeline)?;
		let x = job.run(&context).await;

		if x.is_ok() {
			// Commit only if job ran successfully
			let trans = context.item_db_transaction.into_inner();
			match trans.commit().await {
				Ok(()) => {}
				Err(e) => return Ok(Err(Arc::new(e).into())),
			}
		}

		return Ok(x);
	}
}

impl PipelineRunner {
	/// Get this runner's node dispatcher
	pub fn mut_dispatcher(&mut self) -> &mut NodeDispatcher {
		&mut self.dispatcher
	}
}
