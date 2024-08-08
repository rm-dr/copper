use std::sync::Arc;
use ufo_ds_core::{
	api::Dataset,
	errors::MetastoreError,
	handles::{AttrHandle, ClassHandle},
};
use ufo_pipeline::{
	api::{PipelineData, PipelineNode, PipelineNodeState},
	labels::PipelinePortID,
};

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	nodetype::UFONodeType,
	traits::UFONode,
	UFOContext,
};

pub struct FindItem {
	dataset: Arc<dyn Dataset<UFONodeType>>,
	class: ClassHandle,
	by_attr: AttrHandle,
	attr_type: UFODataStub,

	attr_value: Option<UFOData>,
}

impl FindItem {
	pub fn new(
		ctx: &<Self as PipelineNode>::NodeContext,
		class: ClassHandle,
		by_attr: AttrHandle,
	) -> Result<Self, MetastoreError> {
		let attr_type = ctx.dataset.attr_get_type(by_attr)?.into();
		Ok(FindItem {
			dataset: ctx.dataset.clone(),

			class,
			by_attr,
			attr_type,
			attr_value: None,
		})
	}
}

impl PipelineNode for FindItem {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		assert!(port == 0);
		assert!(data.as_stub() == self.attr_type);
		self.attr_value = Some(data);
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.attr_value.is_none() {
			return Ok(PipelineNodeState::Pending("waiting for input"));
		}

		let found = self.dataset.find_item_with_attr(
			self.by_attr,
			self.attr_value.as_ref().unwrap().as_db_data().unwrap(),
		)?;

		if let Some(item) = found {
			send_data(
				0,
				UFOData::Reference {
					class: self.class,
					item,
				},
			)?;
		} else {
			send_data(
				0,
				UFOData::None(UFODataStub::Reference { class: self.class }),
			)?;
		}

		return Ok(PipelineNodeState::Done);
	}
}

impl UFONode for FindItem {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::FindItem { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: UFODataStub,
	) -> bool {
		match stub {
			UFONodeType::FindItem { .. } => {
				Self::input_default_type(stub, ctx, input_idx) == input_type
			}
			_ => unreachable!(),
		}
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortID,
	) -> Option<usize> {
		match stub {
			UFONodeType::FindItem { .. } => match input_name.id().as_str() {
				"attr_value" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}

	fn input_default_type(stub: &UFONodeType, ctx: &UFOContext, input_idx: usize) -> UFODataStub {
		match stub {
			UFONodeType::FindItem { class, by_attr } => {
				assert!(input_idx == 0);
				let class = ctx.dataset.get_class(&class[..]).unwrap().unwrap();
				let attr = ctx.dataset.get_attr(class, &by_attr).unwrap().unwrap();
				ctx.dataset.attr_get_type(attr).unwrap().into()
			}
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::FindItem { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, ctx: &UFOContext, output_idx: usize) -> UFODataStub {
		match stub {
			UFONodeType::FindItem { class, .. } => {
				assert!(output_idx == 0);
				let class = ctx.dataset.get_class(class).unwrap().unwrap();
				UFODataStub::Reference { class }
			}
			_ => unreachable!(),
		}
	}

	fn output_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		output_name: &PipelinePortID,
	) -> Option<usize> {
		match stub {
			UFONodeType::FindItem { .. } => match output_name.id().as_str() {
				"found_item" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}
}
