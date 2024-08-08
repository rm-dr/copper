use crossbeam::channel::Receiver;
use ufo_metadb::data::{MetaDbData, MetaDbDataStub};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{errors::PipelineError, nodetype::UFONodeType, traits::UFONode, UFOContext};

#[derive(Clone)]
pub struct Noop {
	inputs: Vec<(PipelinePortLabel, MetaDbDataStub, bool)>,
	input_receiver: Receiver<(usize, MetaDbData)>,
}

impl Noop {
	pub fn new(
		_ctx: &<Self as PipelineNode>::NodeContext,
		input_receiver: Receiver<(usize, MetaDbData)>,
		inputs: Vec<(PipelinePortLabel, MetaDbDataStub)>,
	) -> Self {
		Self {
			inputs: inputs.into_iter().map(|(a, b)| (a, b, false)).collect(),
			input_receiver,
		}
	}
}

impl PipelineNode for Noop {
	type NodeContext = UFOContext;
	type DataType = MetaDbData;
	type ErrorType = PipelineError;

	fn take_input<F>(&mut self, send_data: F) -> Result<(), PipelineError>
	where
		F: Fn(usize, MetaDbData) -> Result<(), PipelineError>,
	{
		loop {
			match self.input_receiver.try_recv() {
				Err(crossbeam::channel::TryRecvError::Disconnected)
				| Err(crossbeam::channel::TryRecvError::Empty) => {
					break Ok(());
				}
				Ok((port, data)) => {
					assert!(port < self.inputs.len());
					assert!(!self.inputs[port].2);
					self.inputs[port].2 = true;
					send_data(port, data)?;
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
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		for (_, _, b) in &self.inputs {
			if !b {
				return Ok(PipelineNodeState::Pending("args not ready"));
			}
		}
		return Ok(PipelineNodeState::Done);
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
