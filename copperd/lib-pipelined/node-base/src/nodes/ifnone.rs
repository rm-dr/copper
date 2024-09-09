use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

use crate::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PipelineData,
		PipelinePortID, ProcessSignalError, RunNodeError,
	},
	data::{CopperData, CopperDataStub},
	helpers::ConnectedInput,
};

pub struct IfNone {
	ifnone: ConnectedInput<CopperData>,
	data: ConnectedInput<CopperData>,

	data_type: CopperDataStub,
}

impl IfNone {
	pub fn new(
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<CopperData>>,
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
impl Node<CopperData> for IfNone {
	fn process_signal(&mut self, signal: NodeSignal<CopperData>) -> Result<(), ProcessSignalError> {
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
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		if !(self.data.is_connected() && self.ifnone.is_connected()) {
			return Err(RunNodeError::RequiredInputNotConnected);
		}

		if !(self.data.is_set() && self.ifnone.is_set()) {
			return Ok(NodeState::Pending("args not ready"));
		}

		send_data(
			PipelinePortID::new("out"),
			match &self.data {
				ConnectedInput::Set {
					value: CopperData::None { .. },
				} => self.ifnone.value().unwrap().clone(),
				x => x.value().unwrap().clone(),
			},
		)?;

		Ok(NodeState::Done)
	}
}
