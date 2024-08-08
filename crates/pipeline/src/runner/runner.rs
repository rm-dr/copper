//! Top-level pipeline runner.
//! Runs a set of jobs asyncronously and in parallel.

use std::{collections::VecDeque, fs::File, io::Read, marker::PhantomData, path::Path, sync::Arc};

use super::single::{PipelineSingleJob, SingleJobState};
use crate::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineLabel,
	pipeline::Pipeline,
	syntax::{builder::PipelineBuilder, errors::PipelinePrepareError, spec::PipelineSpec},
	SDataStub, SDataType, SErrorType,
};

/// Pipeline runner configuration
pub struct PipelineRunConfig {
	/// The size of each job's threadpool.
	/// A runner will use at most `node_threads * max_active_jobs` threads.
	pub node_threads: usize,

	/// The maximum number of jobs we'll run at once
	pub max_active_jobs: usize,
}

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner<StubType: PipelineNodeStub> {
	_p: PhantomData<StubType>,
	pipelines: Vec<Arc<Pipeline<StubType>>>,
	context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
	config: PipelineRunConfig,

	active_jobs: Vec<Option<PipelineSingleJob<StubType>>>,
	job_queue: VecDeque<PipelineSingleJob<StubType>>,
}

impl<StubType: PipelineNodeStub> PipelineRunner<StubType> {
	/// Initialize a new runner
	pub fn new(
		config: PipelineRunConfig,
		context: <StubType::NodeType as PipelineNode>::NodeContext,
	) -> Self {
		Self {
			_p: PhantomData,
			pipelines: Vec::new(),
			context: Arc::new(context),

			active_jobs: (0..config.max_active_jobs).map(|_| None).collect(),
			job_queue: VecDeque::new(),

			config,
		}
	}

	/// Load a pipeline into this runner.
	/// A pipeline must be loaded before any jobs can be created.
	pub fn add_pipeline(
		&mut self,
		path: &Path,
		pipeline_name: String,
	) -> Result<(), PipelinePrepareError<SDataStub<StubType>>> {
		let mut f =
			File::open(path).map_err(|error| PipelinePrepareError::CouldNotOpenFile { error })?;

		let mut s: String = Default::default();

		f.read_to_string(&mut s)
			.map_err(|error| PipelinePrepareError::CouldNotReadFile { error })?;

		let spec: PipelineSpec<StubType> = toml::from_str(&s)
			.map_err(|error| PipelinePrepareError::CouldNotParseFile { error })?;

		let built = PipelineBuilder::build(
			self.context.clone(),
			&self.pipelines,
			&pipeline_name[..],
			spec,
		)?;

		self.pipelines.push(Arc::new(built));
		return Ok(());
	}

	/// Get a pipeline that has been added to this runner.
	/// If we don't know of a pipeline with the given named, return `None`.
	pub fn get_pipeline(&self, pipeline_name: &PipelineLabel) -> Option<&Pipeline<StubType>> {
		self.pipelines
			.iter()
			.find(|x| x.name == *pipeline_name)
			.map(|x| &**x)
	}

	/// Add a job to this runner's queue
	pub fn add_job(
		&mut self,

		pipeline_name: &PipelineLabel,
		pipeline_inputs: Vec<SDataType<StubType>>,
	) {
		let pipeline = self
			.pipelines
			.iter()
			.find(|x| x.name == *pipeline_name)
			.unwrap()
			.clone();

		let runner = PipelineSingleJob::new(
			&self.config,
			self.context.clone(),
			pipeline,
			pipeline_inputs,
		);
		self.job_queue.push_back(runner)
	}

	/// Iterate over all active jobs
	pub fn iter_active_jobs(&self) -> impl Iterator<Item = &PipelineSingleJob<StubType>> {
		self.active_jobs.iter().filter_map(|x| x.as_ref())
	}

	/// Update this runner: process all changes that occured since we last called `run()`,
	pub fn run(&mut self) -> Result<(), SErrorType<StubType>> {
		for r in &mut self.active_jobs {
			if let Some(x) = r {
				// Update running jobs
				let s = x.run()?;
				if s == SingleJobState::Done {
					// Drop finished jobs
					r.take();
				}
			} else if self.job_queue.len() != 0 {
				// Start a new job if we have space
				*r = self.job_queue.pop_front();
			}
		}

		Ok(())
	}
}
