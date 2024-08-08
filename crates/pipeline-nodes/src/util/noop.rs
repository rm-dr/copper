use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};
use ufo_storage::data::{StorageData, StorageDataStub};

use crate::{nodetype::UFONodeType, UFOContext, UFONode};

#[derive(Clone)]
pub struct Noop {
	inputs: Vec<(PipelinePortLabel, StorageDataStub)>,
}

impl Noop {
	pub fn new(inputs: Vec<(PipelinePortLabel, StorageDataStub)>) -> Self {
		Self { inputs }
	}
}

impl PipelineNode for Noop {
	type NodeContext = UFOContext;
	type DataType = StorageData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		input: Vec<Self::DataType>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == self.inputs.len());
		for (i, v) in input.into_iter().enumerate() {
			send_data(i, v)?;
		}
		Ok(PipelineNodeState::Done)
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
		input_type: StorageDataStub,
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
	) -> StorageDataStub {
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

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> StorageDataStub {
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
