use smartstring::{LazyCompact, SmartString};
use std::{fs::File, io::Read, marker::PhantomData, path::Path, sync::Arc};

use super::single::{PipelineSingleRunner, SingleRunnerState};
use crate::{
	api::{PipelineData, PipelineNode, PipelineNodeStub},
	errors::PipelineError,
	pipeline::Pipeline,
	syntax::{errors::PipelinePrepareError, spec::PipelineSpec},
};

/// A prepared data processing pipeline.
/// This is guaranteed to be correct:
/// no dependency cycles, no port type mismatch, etc
pub struct PipelineRunner<StubType: PipelineNodeStub> {
	_p: PhantomData<StubType>,
	pipelines: Vec<(SmartString<LazyCompact>, Arc<Pipeline<StubType>>)>,
	node_runners: usize,
}

impl<StubType: PipelineNodeStub> PipelineRunner<StubType> {
	pub fn new(node_runners: usize) -> Self {
		Self {
			_p: PhantomData,
			pipelines: Vec::new(),
			node_runners,
		}
	}

	pub fn add_pipeline(
		&mut self,
		ctx: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		path: &Path,
		pipeline_name: String,
	) -> Result<(), PipelinePrepareError<<<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub>>{
		let mut f =
			File::open(path).map_err(|error| PipelinePrepareError::CouldNotOpenFile { error })?;

		let mut s: String = Default::default();

		f.read_to_string(&mut s)
			.map_err(|error| PipelinePrepareError::CouldNotReadFile { error })?;

		let spec: PipelineSpec<StubType> = toml::from_str(&s)
			.map_err(|error| PipelinePrepareError::CouldNotParseFile { error })?;

		let p = spec.prepare(ctx.clone(), pipeline_name.clone(), &self.pipelines)?;
		self.pipelines.push((pipeline_name.into(), Arc::new(p)));
		return Ok(());
	}

	pub fn get_pipeline(
		&self,
		pipeline_name: SmartString<LazyCompact>,
	) -> Option<Arc<Pipeline<StubType>>> {
		self.pipelines
			.iter()
			.find(|(x, _)| x == &pipeline_name)
			.map(|(_, x)| x.clone())
	}

	pub fn run(
		&self,
		context: Arc<<StubType::NodeType as PipelineNode>::NodeContext>,
		pipeline_name: SmartString<LazyCompact>,
		pipeline_inputs: Vec<<StubType::NodeType as PipelineNode>::DataType>,
	) -> Result<(), PipelineError> {
		let pipeline = self.get_pipeline(pipeline_name).unwrap();

		let mut runner =
			PipelineSingleRunner::new(self.node_runners, context, pipeline, pipeline_inputs);

		let mut s = SingleRunnerState::Running;
		while s == SingleRunnerState::Running {
			s = runner.run()?;
		}

		return Ok(());
	}
}
