use copper_pipelined::base::{NodeDispatcher, PipelineData, PipelineJobContext};
use smartstring::{LazyCompact, SmartString};
use std::collections::{BTreeMap, VecDeque};
use tracing::debug;

use crate::pipeline::job::PipelineJob;

use super::json::PipelineJson;

pub struct PipelineRunnerOptions {
	/// The maximum number of jobs we'll run at once
	/// TODO: rename
	pub max_active_jobs: usize,
}

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner<DataType: PipelineData, ContextType: PipelineJobContext> {
	config: PipelineRunnerOptions,
	dispatcher: NodeDispatcher<DataType, ContextType>,

	/// Jobs that are queued to run
	job_queue: VecDeque<(
		SmartString<LazyCompact>,
		ContextType,
		PipelineJson<DataType>,
		BTreeMap<SmartString<LazyCompact>, DataType>,
	)>,
}

impl<DataType: PipelineData, ContextType: PipelineJobContext>
	PipelineRunner<DataType, ContextType>
{
	/// Initialize a new runner
	pub fn new(config: PipelineRunnerOptions) -> Self {
		Self {
			job_queue: VecDeque::new(),

			config,
			dispatcher: NodeDispatcher::new(),
		}
	}

	/// Add a job to this runner's queue.
	/// Returns the new job's id.
	pub fn add_job(
		&mut self,
		context: ContextType,
		pipeline: PipelineJson<DataType>,
		job_id: &str,
		inputs: BTreeMap<SmartString<LazyCompact>, DataType>,
	) {
		debug!(message = "Adding job", job_id);
		self.job_queue
			.push_back((job_id.into(), context, pipeline, inputs));
	}

	/// Get this runner's node dispatcher
	pub fn mut_dispatcher(&mut self) -> &mut NodeDispatcher<DataType, ContextType> {
		&mut self.dispatcher
	}

	pub async fn run(&mut self) {
		// TODO: no unwrap, don't run a whole node on this call
		if let Some((job_id, context, pipeline, inputs)) = self.job_queue.pop_front() {
			debug!(message = "Running job", ?job_id);
			let x = PipelineJob::<DataType, ContextType>::new(&job_id, inputs, &pipeline).unwrap();

			x.run(&context, &self.dispatcher).await.unwrap();
		}
	}
}
