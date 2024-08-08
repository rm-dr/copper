use crossbeam::channel::Receiver;
use ufo_metadb::data::MetaDbDataStub;
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{
	data::UFOData, errors::PipelineError, nodetype::UFONodeType, traits::UFONode, UFOContext,
};

#[derive(Clone)]
pub struct Print {
	input_receiver: Receiver<(usize, UFOData)>,
	has_received: bool,
}

impl Print {
	pub fn new(
		_ctx: &<Self as PipelineNode>::NodeContext,
		input_receiver: Receiver<(usize, UFOData)>,
	) -> Self {
		Self {
			input_receiver,
			has_received: false,
		}
	}
}

impl PipelineNode for Print {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn take_input<F>(&mut self, _send_data: F) -> Result<(), PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		loop {
			match self.input_receiver.try_recv() {
				Err(crossbeam::channel::TryRecvError::Disconnected)
				| Err(crossbeam::channel::TryRecvError::Empty) => {
					break Ok(());
				}
				Ok((port, data)) => {
					assert!(port == 0);
					println!("{data:?}");
					self.has_received = true;
				}
			}
		}
	}

	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, UFOData) -> Result<(), PipelineError>,
	{
		if self.has_received {
			Ok(PipelineNodeState::Done)
		} else {
			Ok(PipelineNodeState::Pending("args not ready"))
		}
	}
}

impl UFONode for Print {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Print => 1,
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
		_input_type: MetaDbDataStub,
	) -> bool {
		match stub {
			UFONodeType::Print => {
				assert!(input_idx == 0);
				true
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
			UFONodeType::Print => match Into::<&str>::into(input_name) {
				"data" => Some(0),
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
			UFONodeType::Print => {
				assert!(input_idx == 0);
				MetaDbDataStub::Text
			}
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Print => 0,
			_ => unreachable!(),
		}
	}

	fn output_type(_stub: &UFONodeType, _ctx: &UFOContext, _output_idx: usize) -> MetaDbDataStub {
		unreachable!()
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		_output_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::Print => None,
			_ => unreachable!(),
		}
	}
}
