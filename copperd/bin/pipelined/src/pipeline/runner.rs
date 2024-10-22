use copper_jobqueue::id::QueuedJobId;
use copper_pipelined::{
	base::{NodeDispatcher, PipelineData, PipelineJobContext, PipelineJobResult, RunNodeError},
	json::PipelineJson,
};
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, error::Error, fmt::Display};
use tokio::task::{JoinError, JoinSet};
use tracing::debug;

use super::job::PipelineBuildError;
use crate::pipeline::job::PipelineJob;

//
// MARK: Errors
//

#[derive(Debug)]
pub enum StartJobError {
	BuildError(PipelineBuildError),
}

impl Display for StartJobError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::BuildError(_) => write!(f, "pipeline build error"),
		}
	}
}

impl Error for StartJobError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::BuildError(e) => Some(e),
		}
	}
}

impl From<PipelineBuildError> for StartJobError {
	fn from(value: PipelineBuildError) -> Self {
		Self::BuildError(value)
	}
}

//
// MARK: helpers
//

pub enum DoneJobState<ResultType: PipelineJobResult, DataType: PipelineData> {
	Failed {
		job_id: QueuedJobId,
		error: RunNodeError<DataType>,
	},
	Success {
		job_id: QueuedJobId,
		result: ResultType,
	},
}

//
// MARK: Runner
//

pub struct PipelineRunner<
	ResultType: PipelineJobResult,
	DataType: PipelineData,
	ContextType: PipelineJobContext<DataType, ResultType>,
> {
	dispatcher: NodeDispatcher<ResultType, DataType, ContextType>,

	/// Jobs that are running right now
	#[allow(clippy::type_complexity)]
	tasks: JoinSet<(QueuedJobId, Result<ResultType, RunNodeError<DataType>>)>,
}

impl<
		ResultType: PipelineJobResult,
		DataType: PipelineData,
		ContextType: PipelineJobContext<DataType, ResultType>,
	> PipelineRunner<ResultType, DataType, ContextType>
{
	/// Initialize a new runner
	pub fn new() -> Self {
		Self {
			dispatcher: NodeDispatcher::new(),
			tasks: JoinSet::new(),
		}
	}

	/// Start a job in this runner
	pub fn start_job(
		&mut self,
		context: ContextType,
		pipeline: PipelineJson,
		job_id: &QueuedJobId,
		inputs: BTreeMap<SmartString<LazyCompact>, DataType>,
	) -> Result<(), StartJobError> {
		debug!(
			message = "Starting job",
			?job_id,
			running_jobs = self.tasks.len(),
		);

		let job = PipelineJob::<ResultType, DataType, ContextType>::new(
			&self.dispatcher,
			job_id.as_str(),
			inputs.clone(),
			&pipeline,
		)?;

		let job_id_cloned = job_id.clone();
		self.tasks.spawn(async {
			// TODO: handle error
			let x = job.run(context).await;
			(job_id_cloned, x)
		});

		return Ok(());
	}

	/// If any job is done, return its state.
	/// otherwise, return [`None`].
	pub async fn check_done_jobs(
		&mut self,
	) -> Result<Option<DoneJobState<ResultType, DataType>>, JoinError> {
		if let Some(res) = self.tasks.try_join_next() {
			let res = res?;

			if let Err(error) = res.1 {
				return Ok(Some(DoneJobState::Failed {
					job_id: res.0.clone(),
					error,
				}));
			} else {
				return Ok(Some(DoneJobState::Success {
					job_id: res.0.clone(),
					result: res.1.unwrap(),
				}));
			}
		}

		return Ok(None);
	}
}

impl<
		ResultType: PipelineJobResult,
		DataType: PipelineData,
		ContextType: PipelineJobContext<DataType, ResultType>,
	> PipelineRunner<ResultType, DataType, ContextType>
{
	/// Get this runner's node dispatcher
	pub fn mut_dispatcher(&mut self) -> &mut NodeDispatcher<ResultType, DataType, ContextType> {
		&mut self.dispatcher
	}

	/// Return the number of jobs that are currently running
	pub fn n_running_jobs(&self) -> usize {
		self.tasks.len()
	}
}
