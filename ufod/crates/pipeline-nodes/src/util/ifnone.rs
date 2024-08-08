use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortID,
};

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	nodetype::UFONodeType,
	traits::UFONode,
	UFOContext,
};

#[derive(Clone)]
pub struct IfNone {
	ifnone: Option<UFOData>,
	input: Option<UFOData>,
}

impl IfNone {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			ifnone: None,
			input: None,
		}
	}
}

impl PipelineNode for IfNone {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		match port {
			0 => {
				self.input = Some(data);
			}
			1 => {
				self.ifnone = Some(data);
			}
			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.input.is_none() || self.ifnone.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		send_data(
			0,
			match self.input.take().unwrap() {
				UFOData::None(_) => self.ifnone.take().unwrap(),
				x => x,
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
		input_type: UFODataStub,
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
		input_name: &PipelinePortID,
	) -> Option<usize> {
		match stub {
			UFONodeType::IfNone { .. } => match input_name.id().as_str() {
				"data" => Some(0),
				"ifnone" => Some(1),
				_ => None,
			},
			_ => unreachable!(),
		}
	}

	fn input_default_type(stub: &UFONodeType, _ctx: &UFOContext, input_idx: usize) -> UFODataStub {
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

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> UFODataStub {
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
		output_name: &PipelinePortID,
	) -> Option<usize> {
		match stub {
			UFONodeType::IfNone { .. } => match output_name.id().as_str() {
				"out" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}
}
