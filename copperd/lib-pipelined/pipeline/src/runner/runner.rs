//! Top-level pipeline runner.
//! Runs a set of jobs asynchronously and in parallel.

use smartstring::{LazyCompact, SmartString};
use std::{
	collections::{BTreeMap, VecDeque},
	sync::Arc,
};
use tracing::{debug, info, warn};

use super::single::{PipelineSingleJob, PipelineSingleJobError, SingleJobState};
use crate::{
	base::{NodeState, PipelineData, PipelineJobContext},
	dispatcher::NodeDispatcher,
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
};

/// Pipeline runner configuration
pub struct PipelineRunConfig {
	/// The size of each job's threadpool.
	/// A runner will use at most `node_threads * max_active_jobs` threads.
	pub node_threads: usize,

	/// The maximum number of jobs we'll run at once
	pub max_active_jobs: usize,
}

/// A completed pipeline job
#[derive(Debug)]
pub struct CompletedJob<DataType: PipelineData> {
	/// The id of the job that finisehd
	pub job_id: u128,

	/// The name of the pipeline that was run
	pub pipeline: PipelineName,

	/// The arguments this pipeline was run with
	pub input: BTreeMap<SmartString<LazyCompact>, DataType>,

	/// The state of each node when this pipeline finished running
	pub node_states: Vec<(bool, NodeState)>,
}

/// A failed pipeline job
#[derive(Debug)]
pub struct FailedJob<DataType: PipelineData> {
	/// The id of the job that finisehd
	pub job_id: u128,

	/// The name of the pipeline that was run
	pub pipeline: PipelineName,

	/// The arguments this pipeline was run with
	pub input: BTreeMap<SmartString<LazyCompact>, DataType>,

	/// The state of each node when this pipeline finished running
	pub node_states: Vec<(bool, NodeState)>,

	/// The reason this pipeline failed.
	pub error: PipelineSingleJobError,
}

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner<DataType: PipelineData, ContextType: PipelineJobContext<DataType>> {
	config: PipelineRunConfig,
	dispatcher: NodeDispatcher<DataType, ContextType>,

	/// Jobs that are actively running
	active_jobs: Vec<Option<(u128, PipelineSingleJob<DataType, ContextType>)>>,

	/// Jobs that are queued to run
	job_queue: VecDeque<(u128, PipelineSingleJob<DataType, ContextType>)>,

	/// A log of completed jobs
	completed_jobs: VecDeque<CompletedJob<DataType>>,

	/// A log of failed jobs
	failed_jobs: VecDeque<FailedJob<DataType>>,

	/// Job id counter. This will be unique for a long time,
	/// but will eventually wrap back to zero.
	job_id_counter: u128,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
	PipelineRunner<DataType, ContextType>
{
	/// Initialize a new runner
	pub fn new(config: PipelineRunConfig) -> Self {
		Self {
			active_jobs: (0..config.max_active_jobs).map(|_| None).collect(),
			job_queue: VecDeque::new(),
			completed_jobs: VecDeque::new(),
			failed_jobs: VecDeque::new(),
			job_id_counter: 0,

			config,
			dispatcher: NodeDispatcher::new(),
		}
	}

	/// Add a job to this runner's queue.
	/// Returns the new job's id.
	pub fn add_job(
		&mut self,
		context: ContextType,
		pipeline: Arc<Pipeline<DataType, ContextType>>,
	) -> u128 {
		debug!(
			message = "Adding job",
			pipeline = ?pipeline.name,
		);

		let runner = PipelineSingleJob::new(&self.config, context, &self.dispatcher, pipeline);
		self.job_id_counter = self.job_id_counter.wrapping_add(1);
		self.job_queue.push_back((self.job_id_counter, runner));
		return self.job_id_counter;
	}

	/// Iterate over all active jobs
	pub fn iter_active_jobs(
		&self,
	) -> impl Iterator<Item = &(u128, PipelineSingleJob<DataType, ContextType>)> {
		self.active_jobs.iter().filter_map(|x| x.as_ref())
	}

	/// Get this runner's node dispatcher
	pub fn get_dispatcher(&self) -> &NodeDispatcher<DataType, ContextType> {
		&self.dispatcher
	}

	/// Get this runner's node dispatcher
	pub fn mut_dispatcher(&mut self) -> &mut NodeDispatcher<DataType, ContextType> {
		&mut self.dispatcher
	}

	/// Get this runner's job queue
	pub fn get_queued_jobs(&self) -> &VecDeque<(u128, PipelineSingleJob<DataType, ContextType>)> {
		&self.job_queue
	}

	/// Get a list of all jobs this runner has completed
	pub fn get_completed_jobs(&self) -> &VecDeque<CompletedJob<DataType>> {
		&self.completed_jobs
	}

	/// Empty this runner's completed job log
	pub fn clear_completed_jobs(&mut self) {
		self.completed_jobs.clear()
	}

	/// Get a list of all jobs that have failed
	pub fn get_failed_jobs(&self) -> &VecDeque<FailedJob<DataType>> {
		&self.failed_jobs
	}

	/// Empty this runner's failed job log
	pub fn clear_failed_jobs(&mut self) {
		self.failed_jobs.clear()
	}

	/// Find a queued job by id
	pub fn queued_job_by_id(
		&self,
		id: u128,
	) -> Option<&(u128, PipelineSingleJob<DataType, ContextType>)> {
		self.job_queue.iter().find(|(x, _)| *x == id)
	}

	/// Find an active job by id
	pub fn active_job_by_id(
		&self,
		id: u128,
	) -> Option<&(u128, PipelineSingleJob<DataType, ContextType>)> {
		self.active_jobs
			.iter()
			.find(|x| x.as_ref().is_some_and(|(x, _)| *x == id))
			.map(|x| x.as_ref().unwrap())
	}

	/// Update this runner: process all changes that occurred since we last called `run()`,
	pub fn run(&mut self) {
		for r in &mut self.active_jobs {
			if let Some((id, x)) = r {
				// Update running jobs
				match x.run() {
					Ok(SingleJobState::Running) => {}
					Ok(SingleJobState::Done) => {
						info!(
							message = "Job finished",
							job_id = id,
							pipeline = ?x.get_pipeline().name
						);

						// Drop finished jobs
						self.completed_jobs.push_back(CompletedJob {
							job_id: *id,
							pipeline: x.get_pipeline().name.clone(),
							input: x.get_input().clone(),

							node_states: x
								.get_pipeline()
								.iter_node_ids()
								.map(|l| x.get_node_status(l).unwrap())
								.collect(),
						});
						r.take();
					}
					Err(err) => {
						warn!(
							message = "Job failed",
							job_id = id,
							pipeline = ?x.get_pipeline().name,
							in_node = ?err.node,
							error = ?err.error,
						);

						// Drop failed jobs
						self.failed_jobs.push_back(FailedJob {
							job_id: *id,
							pipeline: x.get_pipeline().name.clone(),
							input: x.get_input().clone(),
							error: err,

							node_states: x
								.get_pipeline()
								.iter_node_ids()
								.map(|l| x.get_node_status(l).unwrap())
								.collect(),
						});
						r.take();
					}
				}
			}
		}

		for r in &mut self.active_jobs {
			if r.is_none() && !self.job_queue.is_empty() {
				// Start a new job if we have space
				*r = self.job_queue.pop_front();

				info!(
					message = "Starting job",
					job_id = ?r.as_ref().unwrap().0,
					pipeline = ?r.as_ref().unwrap().1.get_pipeline().name,
				);
			}
		}
	}
}
