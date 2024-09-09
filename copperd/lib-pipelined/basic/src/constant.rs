use pipelined_node_base::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PipelinePortID,
		ProcessSignalError, RunNodeError,
	},
	data::CopperData,
};
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

pub struct Constant {
	value: CopperData,
}

impl Constant {
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
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

impl Node<CopperData> for Constant {
	fn process_signal(&mut self, signal: NodeSignal<CopperData>) -> Result<(), ProcessSignalError> {
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
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		send_data(PipelinePortID::new("out"), self.value.clone())?;
		Ok(NodeState::Done)
	}
}
