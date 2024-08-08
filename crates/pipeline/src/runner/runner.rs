use std::{fs::File, io::Read, marker::PhantomData, path::Path, sync::Arc};

use super::single::{PipelineSingleRunner, SingleRunnerState};
use crate::{
	api::{PipelineNode, PipelineNodeStub},
	errors::PipelineError,
	labels::PipelineLabel,
	pipeline::Pipeline,
	syntax::{builder::PipelineBuilder, errors::PipelinePrepareError, spec::PipelineSpec},
	SDataStub, SDataType,
};

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner<StubType: PipelineNodeStub> {
	_p: PhantomData<StubType>,
	pipelines: Vec<Arc<Pipeline<StubType>>>,
	node_runners: usize,
	context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
}

impl<StubType: PipelineNodeStub> PipelineRunner<StubType> {
	pub fn new(
		context: <StubType::NodeType as PipelineNode>::NodeContext,
		node_runners: usize,
	) -> Self {
		Self {
			_p: PhantomData,
			pipelines: Vec::new(),
			node_runners,
			context: Arc::new(context),
		}
	}

	pub fn add_pipeline(
		&mut self,
		ctx: <StubType::NodeType as PipelineNode>::NodeContext,
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

		let built = PipelineBuilder::build(ctx, &self.pipelines, &pipeline_name[..], spec)?;

		self.pipelines.push(Arc::new(built));
		return Ok(());
	}

	fn get_pipeline(&self, pipeline_name: &PipelineLabel) -> Option<Arc<Pipeline<StubType>>> {
		self.pipelines
			.iter()
			.find(|x| x.name == *pipeline_name)
			.cloned()
	}

	pub fn run(
		&self,
		pipeline_name: &PipelineLabel,
		pipeline_inputs: Vec<SDataType<StubType>>,
	) -> Result<(), PipelineError> {
		let pipeline = self.get_pipeline(pipeline_name).unwrap();

		let mut runner = PipelineSingleRunner::new(
			self.node_runners,
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
