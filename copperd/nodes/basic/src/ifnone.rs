use copper_pipelined::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PipelineData, PortName,
		ProcessSignalError, RunNodeError,
	},
	data::{PipeData, PipeDataStub},
	helpers::ConnectedInput,
};
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

pub struct IfNone {
	ifnone: ConnectedInput<PipeData>,
	data: ConnectedInput<PipeData>,

	data_type: PipeDataStub,
}

impl IfNone {
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 1 {
			return Err(InitNodeError::BadParameterCount { expected: 1 });
		}

		let data_type = if let Some(value) = params.get("value") {
			match value {
				NodeParameterValue::DataType(data_type) => *data_type,
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

		Ok(Self {
			ifnone: ConnectedInput::NotConnected,
			data: ConnectedInput::NotConnected,
			data_type,
		})
	}
}

// Inputs:
// - "data", <T>
// - "ifnone", <T>
// Outputs:
// - "out", <T>
impl Node<PipeData> for IfNone {
	fn process_signal(&mut self, signal: NodeSignal<PipeData>) -> Result<(), ProcessSignalError> {
		match signal {
			NodeSignal::ConnectInput { port } => match port.id().as_str() {
				"data" => self.data.connect(),
				"ifnone" => self.ifnone.connect(),
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::DisconnectInput { port } => match port.id().as_str() {
				"data" => {
					if !self.data.is_connected() {
						unreachable!("disconnected an input that hasn't been connected")
					}
					if !self.data.is_set() {
						return Err(ProcessSignalError::RequiredInputEmpty);
					}
				}
				"ifnone" => {
					if !self.data.is_connected() {
						unreachable!("disconnected an input that hasn't been connected")
					}
					if !self.ifnone.is_set() {
						return Err(ProcessSignalError::RequiredInputEmpty);
					}
				}
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},

			NodeSignal::ReceiveInput { port, data } => match port.id().as_str() {
				"data" => {
					if data.as_stub() != self.data_type {
						return Err(ProcessSignalError::InputWithBadType);
					}

					self.data.set(data);
				}
				"ifnone" => {
					if data.as_stub() != self.data_type {
						return Err(ProcessSignalError::InputWithBadType);
					}

					self.ifnone.set(data);
				}
				_ => return Err(ProcessSignalError::InputPortDoesntExist),
			},
		}

		return Ok(());
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PortName, PipeData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		if !(self.data.is_connected() && self.ifnone.is_connected()) {
			return Err(RunNodeError::RequiredInputNotConnected);
		}

		if !(self.data.is_set() && self.ifnone.is_set()) {
			return Ok(NodeState::Pending("args not ready"));
		}

		send_data(
			PortName::new("out"),
			match &self.data {
				ConnectedInput::Set {
					value: PipeData::None { .. },
				} => self.ifnone.value().unwrap().clone(),
				x => x.value().unwrap().clone(),
			},
		)?;

		Ok(NodeState::Done)
	}
}
