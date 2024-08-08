use ufo_metadb::data::MetaDbDataStub;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{
	data::UFOData, errors::PipelineError, nodetype::UFONodeType, traits::UFONode, UFOContext,
};

#[derive(Clone)]
pub struct Noop {
	received_input: Vec<bool>,
}

impl Noop {
	pub fn new(
		_ctx: &<Self as PipelineNode>::NodeContext,
		inputs: Vec<(PipelinePortLabel, MetaDbDataStub)>,
	) -> Self {
		Self {
			received_input: inputs.into_iter().map(|_| false).collect(),
		}
	}
}

impl PipelineNode for Noop {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input<F>(
		&mut self,
		(port, data): (usize, UFOData),
		send_data: F,
	) -> Result<(), PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(port < self.received_input.len());
		assert!(!self.received_input[port]);
		self.received_input[port] = true;
		send_data(port, data)?;
		return Ok(());
	}

	fn run<F>(&mut self, _send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.received_input.iter().all(|x| *x) {
			return Ok(PipelineNodeState::Done);
		}
		return Ok(PipelineNodeState::Pending("args not ready"));
	}
}

impl UFONode for Noop {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Noop { inputs } => inputs.len(),
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
		input_type: MetaDbDataStub,
	) -> bool {
		match stub {
			UFONodeType::Noop { inputs } => inputs.get(input_idx).unwrap().1 == input_type,
			_ => unreachable!(),
		}
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::Noop { inputs } => inputs
				.iter()
				.enumerate()
				.find(|(_, (n, _))| n == input_name)
				.map(|(x, _)| x),
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> MetaDbDataStub {
		match stub {
			UFONodeType::Noop { inputs } => inputs.get(input_idx).unwrap().1,
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Noop { inputs } => inputs.len(),
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> MetaDbDataStub {
		match stub {
			UFONodeType::Noop { inputs } => inputs.get(output_idx).unwrap().1,
			_ => unreachable!(),
		}
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::Noop { inputs } => inputs
				.iter()
				.enumerate()
				.find(|(_, (n, _))| n == output_name)
				.map(|(x, _)| x),
			_ => unreachable!(),
		}
	}
}
