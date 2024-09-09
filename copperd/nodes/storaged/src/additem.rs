use copper_pipelined::{
	base::{
		InitNodeError, Node, NodeParameterValue, NodeSignal, NodeState, PortName,
		ProcessSignalError, RunNodeError,
	},
	data::PipeData,
	helpers::ConnectedInput,
	CopperContext,
};
use copper_storaged::{
	AttrData, AttributeId, AttributeInfo, ClassId, ClassInfo, Transaction, TransactionAction,
};
use reqwest::StatusCode;
use serde_json::json;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

pub struct AddItem {
	class: ClassId,

	ports: BTreeMap<PortName, AttributeInfo>,
	attrs: BTreeMap<AttributeId, PortName>,
	data: BTreeMap<AttributeId, ConnectedInput<AttrData>>,
}

impl AddItem {
	pub fn new(
		ctx: &CopperContext,
		params: &BTreeMap<SmartString<LazyCompact>, NodeParameterValue<PipeData>>,
	) -> Result<Self, InitNodeError> {
		if params.len() != 1 {
			return Err(InitNodeError::BadParameterCount { expected: 1 });
		}

		let class: ClassId = if let Some(value) = params.get("class") {
			match value {
				NodeParameterValue::Integer(x) => (*x).into(),
				_ => {
					return Err(InitNodeError::BadParameterType {
						param_name: "class".into(),
					})
				}
			}
		} else {
			return Err(InitNodeError::MissingParameter {
				param_name: "class".into(),
			});
		};

		let ports: BTreeMap<PortName, AttributeInfo> = {
			let client = reqwest::blocking::Client::new();

			// TODO: config (state)
			// TODO: handle errors
			// TODO: write mockable client
			let classinfo: ClassInfo = client
				.get(format!("http://localhost:5000/class/{}", u32::from(class)))
				.send()
				.unwrap()
				.json()
				.unwrap();

			classinfo
				.attributes
				.into_iter()
				.map(|x| (PortName::new(x.name.as_str()), x))
				.collect()
		};

		let data = ports
			.iter()
			.map(|(_, v)| (v.id, ConnectedInput::NotConnected))
			.collect();

		let attrs = ports.iter().map(|(k, v)| (v.id, k.clone())).collect();

		Ok(Self {
			class,
			ports,
			data,
			attrs,
		})
	}
}

// Inputs: depends on class
// Outputs: None
impl Node<PipeData> for AddItem {
	fn process_signal(&mut self, signal: NodeSignal<PipeData>) -> Result<(), ProcessSignalError> {
		match signal {
			NodeSignal::ConnectInput { port } => {
				if !self.ports.contains_key(&port) {
					return Err(ProcessSignalError::InputPortDoesntExist);
				}
				let attr = self.ports.get_mut(&port).unwrap();
				self.data.get_mut(&attr.id).unwrap().connect();
			}

			NodeSignal::DisconnectInput { port } => {
				if !self.ports.contains_key(&port) {
					return Err(ProcessSignalError::InputPortDoesntExist);
				}
				let attr = self.ports.get_mut(&port).unwrap();

				if !self.data.get(&attr.id).unwrap().is_connected() {
					unreachable!("port was disconnected before it was connected")
				}

				if !self.data.get(&attr.id).unwrap().is_set() {
					return Err(ProcessSignalError::RequiredInputEmpty);
				}
			}

			NodeSignal::ReceiveInput { port, data } => {
				if !self.ports.contains_key(&port) {
					return Err(ProcessSignalError::InputPortDoesntExist);
				}
				let attr = self.ports.get(&port).unwrap();
				let attr_data = self.data.get_mut(&attr.id).unwrap();

				if !attr_data.is_connected() {
					unreachable!("got input before connecting port")
				}

				if attr_data.is_set() {
					// TODO: no panic.
					// Error when set twice?
					panic!()
				}

				match data {
					PipeData::Blob { .. } => {
						unimplemented!()
					}

					// This should never panic, since we handle panicing cases above.
					x => attr_data.set(x.try_into().unwrap()),
				};
			}
		}

		return Ok(());
	}

	fn run(
		&mut self,
		_send_data: &dyn Fn(PortName, PipeData) -> Result<(), RunNodeError>,
	) -> Result<NodeState, RunNodeError> {
		// Set default values for all disconnected inputs
		for (attr_id, data) in self.data.iter_mut() {
			if !data.is_connected() {
				let attr = self.ports.get(self.attrs.get(attr_id).unwrap()).unwrap();
				data.connect();
				data.set(AttrData::None {
					data_type: attr.data_type,
				})
			}
		}

		// Make sure we've received all data
		for data in self.data.values() {
			if data.is_connected() && !data.is_set() {
				return Ok(NodeState::Pending("waiting for inputs"));
			}
		}

		// Set up transaction
		let transaction = Transaction {
			actions: vec![TransactionAction::AddItem {
				to_class: self.class,
				attributes: self
					.data
					.iter()
					.map(|(k, v)| (k.clone(), v.value().unwrap().clone()))
					.collect(),
			}],
		};

		let client = reqwest::blocking::Client::new();

		// TODO: handle errors
		let res = client
			.post("http://localhost:5000/transaction/apply")
			.json(&json!({
				"transaction": transaction
			}))
			.send()
			.unwrap();

		match res.status() {
			StatusCode::OK => return Ok(NodeState::Done),

			x => {
				panic!("failed with code {x}")
			}
		}
	}
}
