use copper_pipelined::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PortName,
		ProcessSignalError, RunNodeError,
	},
	data::PipeData,
	CopperContext,
};
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

pub struct Constant {
	value: PipeData,
}

impl Constant {
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 1 {
			return Err(InitNodeError::BadParameterCount { expected: 1 });
		}

		let value = if let Some(value) = params.get("value") {
			match value {
				NodeParameterValue::Data(data) => data.clone(),
				_ => {
					return Err(InitNodeError::BadParameterType {
						param_name: "value".into(),
					})
				}
			}
		} else {
			return Err(InitNodeError::MissingParameter {
				param_name: "value".into(),
			});
		};

		Ok(Self { value })
	}
}

impl Node<PipeData, CopperContext> for Constant {
	fn process_signal(
		&mut self,
		_ctx: &CopperContext,
		signal: NodeSignal<PipeData>,
	) -> Result<(), ProcessSignalError> {
		match signal {
			NodeSignal::ConnectInput { .. } => {
				return Err(ProcessSignalError::InputPortDoesntExist)
			}
			NodeSignal::DisconnectInput { .. } => {
				return Err(ProcessSignalError::InputPortDoesntExist)
			}
			NodeSignal::ReceiveInput { .. } => {
				return Err(ProcessSignalError::InputPortDoesntExist)
			}
		}
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn run(
		&mut self,
		_ctx: &CopperContext,
		send_data: &dyn Fn(PortName, PipeData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		send_data(PortName::new("out"), self.value.clone())?;
		Ok(NodeState::Done)
	}
}
