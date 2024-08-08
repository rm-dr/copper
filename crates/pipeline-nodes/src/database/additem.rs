use std::{
	io::Write,
	sync::{Arc, Mutex},
};

use async_broadcast::TryRecvError;
use smartstring::{LazyCompact, SmartString};
use ufo_blobstore::fs::store::{FsBlobHandle, FsBlobStore, FsBlobWriter};
use ufo_metadb::{
	api::{AttrHandle, ClassHandle, MetaDb},
	data::{MetaDbData, MetaDbDataStub},
	errors::MetaDbError,
};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};

use crate::{
	data::UFOData, errors::PipelineError, nodetype::UFONodeType, traits::UFONode, UFOContext,
};

pub struct AddItem {
	db: Arc<Mutex<dyn MetaDb<FsBlobStore>>>,
	class: ClassHandle,
	attrs: Vec<(AttrHandle, SmartString<LazyCompact>, MetaDbDataStub)>,

	data: Vec<Option<DataHold>>,
}

enum DataHold {
	Static(UFOData),
	BlobWriting(
		async_broadcast::Receiver<Arc<Vec<u8>>>,
		Option<FsBlobWriter>,
	),
	BlobDone(FsBlobHandle),
}

impl AddItem {
	pub fn new(
		ctx: &<Self as PipelineNode>::NodeContext,
		class: ClassHandle,
		attrs: Vec<(AttrHandle, SmartString<LazyCompact>, MetaDbDataStub)>,
	) -> Self {
		let data = attrs.iter().map(|_| None).collect();
		AddItem {
			db: ctx.database.clone(),
			class,
			attrs,
			data,
		}
	}
}

impl PipelineNode for AddItem {
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
		assert!(port < self.attrs.len());
		self.data[port] = Some(match data {
			UFOData::Blob { format, data } => {
				let blob = self.db.lock().unwrap().new_blob(&format);
				DataHold::BlobWriting(data, Some(blob))
			}
			x => DataHold::Static(x),
		});
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
		let mut exit = false;
		for i in &mut self.data {
			match i {
				Some(DataHold::BlobWriting(f, buf)) => {
					let mut finish = false;
					loop {
						match f.try_recv() {
							Err(TryRecvError::Closed) => {
								finish = true;
								break;
							}
							Err(TryRecvError::Empty) => {
								exit = true;
								break;
							}
							Err(TryRecvError::Overflowed(_)) => {
								unreachable!()
							}
							Ok(x) => {
								buf.as_mut().unwrap().write(&x[..])?;
							}
						}
					}
					if finish {
						let x = self.db.lock().unwrap().finish_blob(buf.take().unwrap());
						std::mem::swap(i, &mut Some(DataHold::BlobDone(x)));
					}
				}
				Some(_) => {}
				None => exit = true,
			}
		}

		if exit {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		let mut attrs = Vec::new();
		for ((attr, _, _), data) in self.attrs.iter().zip(self.data.iter_mut()) {
			let data = match data.as_ref().unwrap() {
				DataHold::Static(x) => x.as_db_data().unwrap(),
				DataHold::BlobDone(handle) => MetaDbData::Blob {
					handle: handle.clone(),
				},
				_ => unreachable!(),
			};
			attrs.push((*attr, data.into()));
		}
		let res = self.db.lock().unwrap().add_item(self.class, attrs);

		match res {
			Ok(item) => {
				send_data(
					0,
					UFOData::Reference {
						class: self.class,
						item,
					},
				)?;
			}
			Err(err) => match err {
				MetaDbError::UniqueViolated => {
					send_data(
						0,
						UFOData::None(MetaDbDataStub::Reference { class: self.class }),
					)?;
				}
				_ => return Err(err.into()),
			},
		}

		Ok(PipelineNodeState::Done)
	}
}

impl UFONode for AddItem {
	fn n_inputs(stub: &UFONodeType, ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::AddItem { class } => {
				let class = ctx
					.database
					.lock()
					.unwrap()
					.get_class(&class[..])
					.unwrap()
					.unwrap();
				let attrs = ctx.database.lock().unwrap().class_get_attrs(class).unwrap();

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
			UFONodeType::AddItem { .. } => {
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
			UFONodeType::AddItem { class } => {
				let class = ctx
					.database
					.lock()
					.unwrap()
					.get_class(&class[..])
					.unwrap()
					.unwrap();
				let attrs = ctx.database.lock().unwrap().class_get_attrs(class).unwrap();

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
			UFONodeType::AddItem { class } => {
				let class = ctx
					.database
					.lock()
					.unwrap()
					.get_class(&class[..])
					.unwrap()
					.unwrap();
				let attrs = ctx.database.lock().unwrap().class_get_attrs(class).unwrap();

				attrs.into_iter().nth(input_idx).unwrap().2
			}
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::AddItem { class } => {
				let class = ctx
					.database
					.lock()
					.unwrap()
					.get_class(&class[..])
					.unwrap()
					.unwrap();
				let attrs = ctx.database.lock().unwrap().class_get_attrs(class).unwrap();
				attrs.into_iter().count()
			}
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, ctx: &UFOContext, output_idx: usize) -> MetaDbDataStub {
		match stub {
			UFONodeType::AddItem { class } => {
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
			UFONodeType::AddItem { .. } => match Into::<&str>::into(output_name) {
				"added_item" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}
}
