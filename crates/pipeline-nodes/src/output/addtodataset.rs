use smartstring::{LazyCompact, SmartString};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};
use ufo_storage::{
	api::{AttrHandle, ClassHandle},
	data::{StorageData, StorageDataStub},
};

use crate::{nodetype::UFONodeType, UFOContext, UFONode};

pub struct AddToDataset {
	class: ClassHandle,
	attrs: Vec<(AttrHandle, SmartString<LazyCompact>, StorageDataStub)>,
	data: Vec<StorageData>,
}

impl AddToDataset {
	pub fn new(
		class: ClassHandle,
		attrs: Vec<(AttrHandle, SmartString<LazyCompact>, StorageDataStub)>,
	) -> Self {
		AddToDataset {
			class,
			attrs,
			data: Vec::new(),
		}
	}
}

impl PipelineNode for AddToDataset {
	type NodeContext = UFOContext;
	type DataType = StorageData;

	fn init<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		input: Vec<Self::DataType>,
		_send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(input.len() == self.attrs.len());
		self.data = input;
		Ok(PipelineNodeState::Pending)
	}

	fn run<F>(
		&mut self,
		ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		let mut d = ctx.dataset.lock().unwrap();

		let mut attrs = Vec::new();
		for ((attr, _, _), data) in self.attrs.iter().zip(self.data.iter()) {
			attrs.push((*attr, data.clone()));
		}

		let item = d.add_item(self.class, &attrs).unwrap();

		send_data(
			0,
			StorageData::Reference {
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
		input_type: StorageDataStub,
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
	) -> StorageDataStub {
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

	fn output_type(stub: &UFONodeType, ctx: &UFOContext, output_idx: usize) -> StorageDataStub {
		match stub {
			UFONodeType::AddToDataset { class } => {
				assert!(output_idx == 0);
				let mut d = ctx.dataset.lock().unwrap();
				let class = d.get_class(class).unwrap().unwrap();
				StorageDataStub::Reference { class }
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
