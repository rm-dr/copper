use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};
use ufo_storage::data::{StorageData, StorageDataStub};

use crate::{helpers::UFONode, nodetype::UFONodeType, UFOContext};

#[derive(Clone)]
pub struct IfNone {}

impl IfNone {
	pub fn new() -> Self {
		Self {}
	}
}

impl PipelineNode for IfNone {
	type NodeContext = UFOContext;
	type DataType = StorageData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		mut input: Vec<Self::DataType>,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 2);
		let ifnone = input.pop().unwrap();
		let input = input.pop().unwrap();

		send_data(
			0,
			match input {
				StorageData::None(_) => ifnone,
				_ => input,
			},
		)?;

		Ok(PipelineNodeState::Done)
	}
}

impl UFONode for IfNone {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::IfNone { .. } => 2,
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
			UFONodeType::IfNone { data_type } => {
				assert!(input_idx < 2);
				input_type == *data_type
			}
			_ => unreachable!(),
		}
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::IfNone { .. } => match Into::<&str>::into(input_name) {
				"data" => Some(0),
				"ifnone" => Some(1),
				_ => None,
			},
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> StorageDataStub {
		match stub {
			UFONodeType::IfNone { data_type } => {
				assert!(input_idx < 2);
				*data_type
			}
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::IfNone { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> StorageDataStub {
		match stub {
			UFONodeType::IfNone { data_type } => {
				assert!(output_idx == 0);
				*data_type
			}
			_ => unreachable!(),
		}
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::IfNone { .. } => match Into::<&str>::into(output_name) {
				"out" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}
}
