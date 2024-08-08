use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	nodetype::UFONodeType,
	traits::UFONode,
	UFOContext,
};

enum ReceivedInput {
	NotReceived,
	Received(UFOData),
	Sent,
}

pub struct Noop {
	received_input: Vec<ReceivedInput>,
}

impl Noop {
	pub fn new(
		_ctx: &<Self as PipelineNode>::NodeContext,
		inputs: Vec<(PipelinePortLabel, UFODataStub)>,
	) -> Self {
		Self {
			received_input: inputs
				.into_iter()
				.map(|_| ReceivedInput::NotReceived)
				.collect(),
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

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		assert!(port < self.received_input.len());
		assert!(matches!(
			self.received_input[port],
			ReceivedInput::NotReceived
		));
		self.received_input[port] = ReceivedInput::Received(data);
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		let mut is_done = false;
		for i in 0..self.received_input.len() {
			match self.received_input[i] {
				ReceivedInput::NotReceived => {
					is_done = false;
				}
				ReceivedInput::Received(_) => {
					let d = std::mem::replace(&mut self.received_input[i], ReceivedInput::Sent);
					send_data(
						i,
						match d {
							ReceivedInput::Received(d) => d,
							_ => unreachable!(),
						},
					)?;
				}
				ReceivedInput::Sent => {}
			}
		}

		if is_done {
			return Ok(PipelineNodeState::Done);
		} else {
			return Ok(PipelineNodeState::Pending("waiting for args"));
		}
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
		input_type: UFODataStub,
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

	fn input_default_type(stub: &UFONodeType, _ctx: &UFOContext, input_idx: usize) -> UFODataStub {
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

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> UFODataStub {
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
