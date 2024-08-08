use std::sync::{Arc, Mutex};
use ufo_blobstore::fs::store::FsBlobStore;
use ufo_metadb::{
	api::{AttrHandle, ClassHandle, MetaDb},
	data::MetaDbDataStub,
	errors::MetaDbError,
};
use ufo_pipeline::{
	api::{PipelineData, PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{
	data::UFOData, errors::PipelineError, nodetype::UFONodeType, traits::UFONode, UFOContext,
};

pub struct FindItem {
	db: Arc<Mutex<dyn MetaDb<FsBlobStore>>>,
	class: ClassHandle,
	by_attr: AttrHandle,
	attr_type: MetaDbDataStub,

	attr_value: Option<UFOData>,
}

impl FindItem {
	pub fn new(
		ctx: &<Self as PipelineNode>::NodeContext,
		class: ClassHandle,
		by_attr: AttrHandle,
	) -> Result<Self, MetaDbError> {
		let attr_type = ctx.database.lock().unwrap().attr_get_type(by_attr)?;
		Ok(FindItem {
			db: ctx.database.clone(),
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

	fn take_input<F>(
		&mut self,
		(port, data): (usize, UFOData),
		_send_data: F,
	) -> Result<(), PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		assert!(port == 0);
		assert!(data.as_stub() == self.attr_type);
		self.attr_value = Some(data);
		return Ok(());
	}

	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.attr_value.is_none() {
			return Ok(PipelineNodeState::Pending("waiting for input"));
		}

		let found = self.db.lock().unwrap().find_item_with_attr(
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
				UFOData::None(MetaDbDataStub::Reference { class: self.class }),
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
		input_type: MetaDbDataStub,
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
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::FindItem { .. } => match Into::<&str>::into(input_name) {
				"attr_value" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
	) -> MetaDbDataStub {
		match stub {
			UFONodeType::FindItem { class, by_attr } => {
				assert!(input_idx == 0);
				let mut d = ctx.database.lock().unwrap();
				let class = d.get_class(&class[..]).unwrap().unwrap();
				let attr = d.get_attr(class, &by_attr).unwrap().unwrap();
				d.attr_get_type(attr).unwrap()
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

	fn output_type(stub: &UFONodeType, ctx: &UFOContext, output_idx: usize) -> MetaDbDataStub {
		match stub {
			UFONodeType::FindItem { class, .. } => {
				assert!(output_idx == 0);
				let mut d = ctx.database.lock().unwrap();
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
			UFONodeType::FindItem { .. } => match Into::<&str>::into(output_name) {
				"found_item" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}
}
