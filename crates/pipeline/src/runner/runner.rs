//! Top-level pipeline runner.
//! Runs a set of jobs asyncronously and in parallel.

use std::{collections::VecDeque, marker::PhantomData, sync::Arc};

use tracing::debug;

use super::single::{PipelineSingleJob, SingleJobState};
use crate::{
	api::{PipelineNode, PipelineNodeState, PipelineNodeStub},
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
	SDataType, SErrorType,
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
pub struct CompletedJob<NodeStubType: PipelineNodeStub> {
	/// The id of the job that finisehd
	pub job_id: u128,

	/// The name of the pipeline that was run
	pub pipeline: PipelineName,

	/// The arguments this pipeline was run with
	pub input: Vec<SDataType<NodeStubType>>,

	/// The state of each node when this pipeline finished running
	pub node_states: Vec<(bool, PipelineNodeState)>,
}

/// A failed pipeline job
#[derive(Debug)]
pub struct FailedJob<NodeStubType: PipelineNodeStub> {
	/// The id of the job that finisehd
	pub job_id: u128,

	/// The name of the pipeline that was run
	pub pipeline: PipelineName,

	/// The arguments this pipeline was run with
	pub input: Vec<SDataType<NodeStubType>>,

	/// The state of each node when this pipeline finished running
	pub node_states: Vec<(bool, PipelineNodeState)>,

	/// The reason this pipeline failed.
	pub error: SErrorType<NodeStubType>,
}

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner<NodeStubType: PipelineNodeStub> {
	_p: PhantomData<NodeStubType>,
	context: Arc<<NodeStubType::NodeType as PipelineNode>::NodeContext>,
	config: PipelineRunConfig,

	/// Jobs that are actively running
	active_jobs: Vec<Option<(u128, PipelineSingleJob<NodeStubType>)>>,

	/// Jobs that are queued to run
	job_queue: VecDeque<(u128, PipelineSingleJob<NodeStubType>)>,

	/// A log of completed jobs
	completed_jobs: VecDeque<CompletedJob<NodeStubType>>,

	/// A log of failed jobs
	failed_jobs: VecDeque<FailedJob<NodeStubType>>,

	/// Job id counter. This will be unique for a long time,
	/// but will eventually wrap back to zero.
	job_id_counter: u128,
}

impl<NodeStubType: PipelineNodeStub> PipelineRunner<NodeStubType> {
	/// Initialize a new runner
	pub fn new(
		config: PipelineRunConfig,
		context: <NodeStubType::NodeType as PipelineNode>::NodeContext,
	) -> Self {
		Self {
			_p: PhantomData,
			context: Arc::new(context),

			active_jobs: (0..config.max_active_jobs).map(|_| None).collect(),
			job_queue: VecDeque::new(),
			completed_jobs: VecDeque::new(),
			failed_jobs: VecDeque::new(),

			job_id_counter: 0,
			config,
		}
	}

	/// Get this runner's context
	pub fn get_context(&self) -> &Arc<<NodeStubType::NodeType as PipelineNode>::NodeContext> {
		&self.context
	}

	/// Add a job to this runner's queue.
	/// Returns the new job's id.
	pub fn add_job(
		&mut self,
		pipeline: Arc<Pipeline<NodeStubType>>,
		pipeline_inputs: Vec<SDataType<NodeStubType>>,
	) -> u128 {
		debug!(
			source = "runner",
			summary = "Adding job",
			pipeline = ?pipeline.get_name()
		);

		let runner = PipelineSingleJob::new(
			&self.config,
			self.context.clone(),
			pipeline,
			pipeline_inputs,
		);
		self.job_id_counter = self.job_id_counter.wrapping_add(1);
		self.job_queue.push_back((self.job_id_counter, runner));
		return self.job_id_counter;
	}

	/// Iterate over all active jobs
	pub fn iter_active_jobs(
		&self,
	) -> impl Iterator<Item = &(u128, PipelineSingleJob<NodeStubType>)> {
		self.active_jobs.iter().filter_map(|x| x.as_ref())
	}

	/// Get this runner's job queue
	pub fn get_queued_jobs(&self) -> &VecDeque<(u128, PipelineSingleJob<NodeStubType>)> {
		&self.job_queue
	}

	/// Get a list of all jobs this runner has completed
	pub fn get_completed_jobs(&self) -> &VecDeque<CompletedJob<NodeStubType>> {
		&self.completed_jobs
	}

	/// Empty this runner's completed job log
	pub fn clear_completed_jobs(&mut self) {
		self.completed_jobs.clear()
	}

	/// Get a list of all jobs that have failed
	pub fn get_failed_jobs(&self) -> &VecDeque<FailedJob<NodeStubType>> {
		&self.failed_jobs
	}

	/// Empty this runner's failed job log
	pub fn clear_failed_jobs(&mut self) {
		self.failed_jobs.clear()
	}

	/// Find a queued job by id
	pub fn queued_job_by_id(&self, id: u128) -> Option<&(u128, PipelineSingleJob<NodeStubType>)> {
		self.job_queue.iter().find(|(x, _)| *x == id)
	}

	/// Find an active job by id
	pub fn active_job_by_id(&self, id: u128) -> Option<&(u128, PipelineSingleJob<NodeStubType>)> {
		self.active_jobs
			.iter()
			.find(|x| x.as_ref().is_some_and(|(x, _)| *x == id))
			.map(|x| x.as_ref().unwrap())
	}

	/// Update this runner: process all changes that occured since we last called `run()`,
	pub fn run(&mut self) -> Result<(), SErrorType<NodeStubType>> {
		for r in &mut self.active_jobs {
			if let Some((id, x)) = r {
				// Update running jobs
				match x.run() {
					Ok(SingleJobState::Running) => {}
					Ok(SingleJobState::Done) => {
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
			if r.is_none() {
				// Start a new job if we have space
				*r = self.job_queue.pop_front();
			}
		}

		Ok(())
	}
}
