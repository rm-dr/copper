//! Top-level pipeline runner.
//! Asynchronously Runs a set of pipelines on multiple threads.

use std::{fs::File, io::Read, marker::PhantomData, path::Path, sync::Arc};

use super::single::{PipelineSingleRunner, SingleRunnerState};
use crate::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineLabel,
	pipeline::Pipeline,
	syntax::{builder::PipelineBuilder, errors::PipelinePrepareError, spec::PipelineSpec},
	SDataStub, SDataType, SErrorType,
};

/// Pipeline runner configuration
pub struct PipelineRunConfig {
	/// The number of threads to use to run nodes in each pipeline
	pub node_threads: usize,
}

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner<StubType: PipelineNodeStub> {
	_p: PhantomData<StubType>,
	pipelines: Vec<Arc<Pipeline<StubType>>>,
	context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
	config: PipelineRunConfig,
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
			config,
			context: Arc::new(context),
		}
	}

	/// Load a pipeline into this runner.
	///
	/// A pipeline must be loaded before any instances of it are run.
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

	fn get_pipeline(&self, pipeline_name: &PipelineLabel) -> Option<Arc<Pipeline<StubType>>> {
		self.pipelines
			.iter()
			.find(|x| x.name == *pipeline_name)
			.cloned()
	}

	/// Run a pipeline with the given inputs
	pub fn run(
		&self,
		pipeline_name: &PipelineLabel,
		pipeline_inputs: Vec<SDataType<StubType>>,
	) -> Result<(), SErrorType<StubType>> {
		let pipeline = self.get_pipeline(pipeline_name).unwrap();

		let mut runner = PipelineSingleRunner::new(
			&self.config,
			self.context.clone(),
			pipeline,
			pipeline_inputs,
		);

		let mut s = SingleRunnerState::Running;
		while s == SingleRunnerState::Running {
			s = runner.run()?;
		}

		return Ok(());
	}
}
