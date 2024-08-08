use ufo_metadb::data::MetaDbDataStub;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{
	data::UFOData, errors::PipelineError, nodetype::UFONodeType, traits::UFONode, UFOContext,
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

	fn take_input<F>(
		&mut self,
		(port, data): (usize, UFOData),
		_send_data: F,
	) -> Result<(), PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
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
		input_type: MetaDbDataStub,
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
	) -> MetaDbDataStub {
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

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> MetaDbDataStub {
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
