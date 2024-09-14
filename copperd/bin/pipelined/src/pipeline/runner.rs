use copper_pipelined::base::{NodeDispatcher, PipelineData, PipelineJobContext, RunNodeError};
use smartstring::{LazyCompact, SmartString};
use std::{
	collections::{BTreeMap, VecDeque},
	sync::Arc,
};
use tokio::task::{JoinError, JoinSet};
use tracing::debug;

use super::json::PipelineJson;
use crate::pipeline::job::PipelineJob;

pub struct PipelineRunnerOptions {
	/// The maximum number of jobs we'll run at once
	pub max_running_jobs: usize,
}

struct JobEntry<DataType: PipelineData, ContextType: PipelineJobContext> {
	id: SmartString<LazyCompact>,
	context: Arc<ContextType>,
	pipeline: PipelineJson<DataType>,
	inputs: BTreeMap<SmartString<LazyCompact>, DataType>,
}

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner<DataType: PipelineData, ContextType: PipelineJobContext> {
	config: PipelineRunnerOptions,
	dispatcher: NodeDispatcher<DataType, ContextType>,

	/// Jobs that are queued to run
	job_queue: VecDeque<JobEntry<DataType, ContextType>>,

	/// Jobs that are running right now
	running_jobs: JoinSet<(
		JobEntry<DataType, ContextType>,
		Result<(), RunNodeError<DataType>>,
	)>,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext>
	PipelineRunner<DataType, ContextType>
{
	/// Initialize a new runner
	pub fn new(config: PipelineRunnerOptions) -> Self {
		Self {
			config,
			dispatcher: NodeDispatcher::new(),

			job_queue: VecDeque::new(),
			running_jobs: JoinSet::new(),
		}
	}

	/// Add a job to this runner's queue
	pub fn add_job(
		&mut self,
		context: ContextType,
		pipeline: PipelineJson<DataType>,
		job_id: &str,
		inputs: BTreeMap<SmartString<LazyCompact>, DataType>,
	) {
		debug!(message = "Adding job to queue", job_id);
		self.job_queue.push_back(JobEntry {
			id: job_id.into(),
			context: Arc::new(context),
			pipeline,
			inputs,
		});
	}

	/// Get this runner's node dispatcher
	pub fn mut_dispatcher(&mut self) -> &mut NodeDispatcher<DataType, ContextType> {
		&mut self.dispatcher
	}

	pub async fn run(&mut self) -> Result<(), JoinError> {
		//
		// Process finished jobs
		//
		while let Some(res) = self.running_jobs.try_join_next() {
			let res = res?;

			if let Err(err) = res.1 {
				debug!(
					message = "Job failed",
					job_id = ?res.0.id,
					error = format!("{err:?}")
				);
			} else {
				debug!(
					message = "Job finished with no errors",
					job_id = ?res.0.id,
				);
			}
		}

		//
		// Start new jobs, if there is space in the set
		// and jobs in the queue.
		//
		while self.running_jobs.len() < self.config.max_running_jobs && !self.job_queue.is_empty() {
			let queued_job = self.job_queue.pop_front().unwrap();

			debug!(
				message = "Starting job",
				job_id = ?queued_job.id,
				running_jobs = self.running_jobs.len(),
				max_running_jobs = self.config.max_running_jobs,
				queued_jobs = self.job_queue.len()
			);

			let job = PipelineJob::<DataType, ContextType>::new(
				&self.dispatcher,
				&queued_job.id,
				queued_job.inputs.clone(),
				&queued_job.pipeline,
			)
			.unwrap();

			self.running_jobs.spawn(async {
				let x = job.run(queued_job.context.clone()).await;
				(queued_job, x)
			});
		}

		return Ok(());
	}
}
