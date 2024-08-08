use crossbeam::channel::Receiver;
use ufo_metadb::data::{MetaDbData, MetaDbDataStub};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{errors::PipelineError, nodetype::UFONodeType, traits::UFONode, UFOContext};

#[derive(Clone)]
pub struct IfNone {
	input_receiver: Receiver<(usize, MetaDbData)>,
	ifnone: Option<MetaDbData>,
	input: Option<MetaDbData>,
}

impl IfNone {
	pub fn new(
		_ctx: &<Self as PipelineNode>::NodeContext,
		input_receiver: Receiver<(usize, MetaDbData)>,
	) -> Self {
		Self {
			input_receiver,
			ifnone: None,
			input: None,
		}
	}
}

impl PipelineNode for IfNone {
	type NodeContext = UFOContext;
	type DataType = MetaDbData;
	type ErrorType = PipelineError;

	fn take_input<F>(&mut self, _send_data: F) -> Result<(), PipelineError>
	where
		F: Fn(usize, MetaDbData) -> Result<(), PipelineError>,
	{
		loop {
			match self.input_receiver.try_recv() {
				Err(crossbeam::channel::TryRecvError::Disconnected)
				| Err(crossbeam::channel::TryRecvError::Empty) => {
					break Ok(());
				}
				Ok((port, data)) => match port {
					0 => {
						self.input = Some(data);
					}
					1 => {
						self.ifnone = Some(data);
					}
					_ => unreachable!(),
				},
			}
		}
	}

	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.input.is_none() || self.ifnone.is_none() {
			return Ok(PipelineNodeState::Pending);
		}

		send_data(
			0,
			match self.input.take().unwrap() {
				MetaDbData::None(_) => self.ifnone.take().unwrap(),
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
