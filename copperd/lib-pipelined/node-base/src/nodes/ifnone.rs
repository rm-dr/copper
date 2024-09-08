use std::collections::BTreeMap;

use pipelined_pipeline::{
	api::{InitNodeError, Node, NodeInfo, NodeState, PipelineData, RunNodeError},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};
use smartstring::{LazyCompact, SmartString};

use crate::data::{CopperData, CopperDataStub};

pub struct IfNone {
	inputs: BTreeMap<PipelinePortID, CopperDataStub>,
	outputs: BTreeMap<PipelinePortID, CopperDataStub>,

	ifnone: Option<CopperData>,
	input: Option<CopperData>,
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
			inputs: BTreeMap::from([
				(PipelinePortID::new("data"), data_type),
				(PipelinePortID::new("ifnone"), data_type),
			]),

			outputs: BTreeMap::from([(PipelinePortID::new("out"), data_type)]),

			ifnone: None,
			input: None,
		})
	}
}

impl NodeInfo<CopperData> for IfNone {
	fn inputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
		&self.inputs
	}

	fn outputs(&self) -> &BTreeMap<PipelinePortID, <CopperData as PipelineData>::DataStubType> {
		&self.outputs
	}
}

impl Node<CopperData> for IfNone {
	fn get_info(&self) -> &dyn NodeInfo<CopperData> {
		self
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(
		&mut self,
		target_port: PipelinePortID,
		input_data: CopperData,
	) -> Result<(), RunNodeError> {
		match target_port.id().as_str() {
			"data" => {
				self.input = Some(input_data);
			}
			"ifnone" => {
				self.ifnone = Some(input_data);
			}
			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(PipelinePortID, CopperData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		if self.input.is_none() || self.ifnone.is_none() {
			return Ok(NodeState::Pending("args not ready"));
		}

		send_data(
			PipelinePortID::new("out"),
			match self.input.take().unwrap() {
				CopperData::None { .. } => self.ifnone.take().unwrap(),
				x => x,
			},
		)?;

		Ok(NodeState::Done)
	}
}
