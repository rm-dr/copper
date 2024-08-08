use std::collections::BTreeMap;

use smartstring::{LazyCompact, SmartString};
use ufo_pipeline::{
	api::{
		NodeInputInfo, NodeOutputInfo, PipelineData, PipelineNode, PipelineNodeError,
		PipelineNodeState,
	},
	dispatcher::NodeParameterValue,
	labels::PipelinePortID,
};

use crate::{data::UFOData, UFOContext};

pub struct IfNone {
	inputs: Vec<NodeInputInfo<<UFOData as PipelineData>::DataStubType>>,
	outputs: Vec<NodeOutputInfo<<UFOData as PipelineData>::DataStubType>>,

	ifnone: Option<UFOData>,
	input: Option<UFOData>,
}

impl IfNone {
	pub fn new(
		_ctx: &UFOContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<UFOData>>,
	) -> Result<Self, PipelineNodeError> {
		if params.len() != 1 {
			return Err(PipelineNodeError::BadParameterCount { expected: 1 });
		}

		let data_type = if let Some(value) = params.get("value") {
			match value {
				NodeParameterValue::DataType(data_type) => data_type.clone(),
				_ => {
					return Err(PipelineNodeError::BadParameterType {
						param_name: "value".into(),
					})
				}
			}
		} else {
			return Err(PipelineNodeError::MissingParameter {
				param_name: "value".into(),
			});
		};

		Ok(Self {
			inputs: vec![
				NodeInputInfo {
					name: PipelinePortID::new("data"),
					accepts_type: data_type,
				},
				NodeInputInfo {
					name: PipelinePortID::new("ifnone"),
					accepts_type: data_type,
				},
			],

			outputs: vec![NodeOutputInfo {
				name: PipelinePortID::new("out"),
				produces_type: data_type,
			}],

			ifnone: None,
			input: None,
		})
	}
}

impl PipelineNode<UFOData> for IfNone {
	fn inputs(&self) -> &[NodeInputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.inputs
	}

	fn outputs(&self) -> &[NodeOutputInfo<<UFOData as PipelineData>::DataStubType>] {
		&self.outputs
	}

	fn quick_run(&self) -> bool {
		true
	}

	fn take_input(
		&mut self,
		target_port: usize,
		input_data: UFOData,
	) -> Result<(), PipelineNodeError> {
		match target_port {
			0 => {
				self.input = Some(input_data);
			}
			1 => {
				self.ifnone = Some(input_data);
			}
			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run(
		&mut self,
		send_data: &dyn Fn(usize, UFOData) -> Result<(), PipelineNodeError>,
	) -> Result<PipelineNodeState, PipelineNodeError> {
		if self.input.is_none() || self.ifnone.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		send_data(
			0,
			match self.input.take().unwrap() {
				UFOData::None { .. } => self.ifnone.take().unwrap(),
				x => x,
			},
		)?;

		Ok(PipelineNodeState::Done)
	}
}
