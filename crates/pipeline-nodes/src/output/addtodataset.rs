use crossbeam::channel::Receiver;
use smartstring::{LazyCompact, SmartString};
use ufo_metadb::{
	api::{AttrHandle, ClassHandle},
	data::{MetaDbData, MetaDbDataStub},
};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};

use crate::{nodetype::UFONodeType, traits::UFONode, UFOContext};

pub struct AddToDataset {
	class: ClassHandle,
	attrs: Vec<(AttrHandle, SmartString<LazyCompact>, MetaDbDataStub)>,
	data: Vec<Option<MetaDbData>>,
	input_receiver: Receiver<(usize, MetaDbData)>,
}

impl AddToDataset {
	pub fn new(
		input_receiver: Receiver<(usize, MetaDbData)>,
		class: ClassHandle,
		attrs: Vec<(AttrHandle, SmartString<LazyCompact>, MetaDbDataStub)>,
	) -> Self {
		let data = attrs.iter().map(|_| None).collect();
		AddToDataset {
			input_receiver,
			class,
			attrs,
			data,
		}
	}
}

impl PipelineNode for AddToDataset {
	type NodeContext = UFOContext;
	type DataType = MetaDbData;

	fn take_input<F>(&mut self, _send_data: F) -> Result<(), PipelineError>
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
					assert!(port < self.attrs.len());
					self.data[port] = Some(data);
				}
			}
		}
	}

	fn run<F>(
		&mut self,
		ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.data.iter().any(|x| x.is_none()) {
			return Ok(PipelineNodeState::Pending);
		}

		let mut d = ctx.dataset.lock().unwrap();

		let mut attrs = Vec::new();
		for ((attr, _, _), data) in self.attrs.iter().zip(self.data.iter_mut()) {
			attrs.push((*attr, data.take().unwrap()));
		}
		let item = d.add_item(self.class, attrs).unwrap();

		send_data(
			0,
			MetaDbData::Reference {
				class: self.class,
				item,
			},
		)?;

		Ok(PipelineNodeState::Done)
	}
}

impl UFONode for AddToDataset {
	fn n_inputs(stub: &UFONodeType, ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::AddToDataset { class } => {
				let class = ctx
					.dataset
					.lock()
					.unwrap()
					.get_class(&class[..])
					.unwrap()
					.unwrap();
				let attrs = ctx.dataset.lock().unwrap().class_get_attrs(class).unwrap();

				attrs.into_iter().count()
			}
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: MetaDbDataStub,
	) -> bool {
		match stub {
			UFONodeType::AddToDataset { .. } => {
				Self::input_default_type(stub, ctx, input_idx) == input_type
			}
			_ => unreachable!(),
		}
	}

	fn input_with_name(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::AddToDataset { class } => {
				let class = ctx
					.dataset
					.lock()
					.unwrap()
					.get_class(&class[..])
					.unwrap()
					.unwrap();
				let attrs = ctx.dataset.lock().unwrap().class_get_attrs(class).unwrap();

				attrs
					.into_iter()
					.enumerate()
					.find(|(_, (_, name, _))| PipelinePortLabel::from(name) == *input_name)
					.map(|(i, _)| i)
			}
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
	) -> MetaDbDataStub {
		match stub {
			UFONodeType::AddToDataset { class } => {
				let class = ctx
					.dataset
					.lock()
					.unwrap()
					.get_class(&class[..])
					.unwrap()
					.unwrap();
				let attrs = ctx.dataset.lock().unwrap().class_get_attrs(class).unwrap();

				attrs.into_iter().nth(input_idx).unwrap().2
			}
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::AddToDataset { class } => {
				let class = ctx
					.dataset
					.lock()
					.unwrap()
					.get_class(&class[..])
					.unwrap()
					.unwrap();
				let attrs = ctx.dataset.lock().unwrap().class_get_attrs(class).unwrap();
				attrs.into_iter().count()
			}
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, ctx: &UFOContext, output_idx: usize) -> MetaDbDataStub {
		match stub {
			UFONodeType::AddToDataset { class } => {
				assert!(output_idx == 0);
				let mut d = ctx.dataset.lock().unwrap();
				let class = d.get_class(class).unwrap().unwrap();
				MetaDbDataStub::Reference { class }
			}
			_ => unreachable!(),
		}
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::AddToDataset { .. } => match Into::<&str>::into(output_name) {
				"added_item" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}
}
