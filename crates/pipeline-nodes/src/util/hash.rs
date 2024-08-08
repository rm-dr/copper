use async_broadcast::TryRecvError;
use crossbeam::channel::Receiver;
use sha2::{Digest, Sha256, Sha512};
use std::sync::Arc;
use ufo_metadb::data::{HashType, MetaDbData, MetaDbDataStub};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	errors::PipelineError,
	labels::PipelinePortLabel,
};

use crate::{nodetype::UFONodeType, traits::UFONode, UFOContext};

#[derive(Clone)]
pub struct Hash {
	data: Option<MetaDbData>,
	hash_type: HashType,

	// TODO: write directly to hasher
	buffer: Vec<u8>,

	input_receiver: Receiver<(usize, MetaDbData)>,
}

impl Hash {
	pub fn new(
		_ctx: &<Self as PipelineNode>::NodeContext,
		input_receiver: Receiver<(usize, MetaDbData)>,
		hash_type: HashType,
	) -> Self {
		Self {
			data: None,
			hash_type,
			buffer: Vec::new(),
			input_receiver,
		}
	}
}

impl PipelineNode for Hash {
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
				Ok((port, data)) => match port {
					0 => {
						self.data = Some(data);
					}
					_ => unreachable!("bad input port {port}"),
				},
			}
		}
	}

	fn run<F>(
		&mut self,
		_ctx: &Self::NodeContext,
		send_data: F,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.data.is_none() {
			return Ok(PipelineNodeState::Pending);
		}

		let result = match self.data.as_mut().unwrap() {
			MetaDbData::Binary { data, .. } => match self.hash_type {
				HashType::MD5 => md5::compute(&**data).to_vec(),
				HashType::SHA256 => {
					let mut hasher = Sha256::new();
					hasher.update(&**data);
					hasher.finalize().to_vec()
				}
				HashType::SHA512 => {
					let mut hasher = Sha512::new();
					hasher.update(&**data);
					hasher.finalize().to_vec()
				}
			},
			MetaDbData::Blob { data, .. } => match self.hash_type {
				HashType::MD5 => loop {
					match data.try_recv() {
						Err(TryRecvError::Closed) => {
							let mut context = md5::Context::new();
							context.consume(&self.buffer);
							break context.compute().to_vec();
						}
						Err(TryRecvError::Empty) => {
							return Ok(PipelineNodeState::Pending);
						}
						Err(_) => panic!(),
						Ok(x) => {
							self.buffer.extend(&*x);
						}
					}
				},
				HashType::SHA256 => loop {
					match data.try_recv() {
						Err(TryRecvError::Closed) => {
							let mut hasher = Sha256::new();
							hasher.update(&self.buffer);
							break hasher.finalize().to_vec();
						}
						Err(TryRecvError::Empty) => {
							return Ok(PipelineNodeState::Pending);
						}
						Err(_) => panic!(),
						Ok(x) => {
							self.buffer.extend(&*x);
						}
					}
				},
				HashType::SHA512 => loop {
					match data.try_recv() {
						Err(TryRecvError::Closed) => {
							let mut hasher = Sha256::new();
							hasher.update(&self.buffer);
							break hasher.finalize().to_vec();
						}
						Err(TryRecvError::Empty) => {
							return Ok(PipelineNodeState::Pending);
						}
						Err(_) => panic!(),
						Ok(x) => {
							self.buffer.extend(&*x);
						}
					}
				},
			},
			_ => todo!(),
		};

		send_data(
			0,
			MetaDbData::Hash {
				format: self.hash_type,
				data: Arc::new(result),
			},
		)?;

		return Ok(PipelineNodeState::Done);
	}
}

impl UFONode for Hash {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Hash { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
		input_type: MetaDbDataStub,
	) -> bool {
		match stub {
			UFONodeType::Hash { .. } => {
				assert!(input_idx < 1);
				match input_type {
					MetaDbDataStub::Blob | MetaDbDataStub::Binary => true,
					_ => false,
				}
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
			UFONodeType::Hash { .. } => match Into::<&str>::into(input_name) {
				"data" => Some(0),
				_ => None,
			},
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> MetaDbDataStub {
		match stub {
			UFONodeType::Hash { .. } => {
				assert!(input_idx < 1);
				MetaDbDataStub::Binary
			}
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::Hash { .. } => 1,
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> MetaDbDataStub {
		match stub {
			UFONodeType::Hash { hash_type } => {
				assert!(output_idx == 0);
				MetaDbDataStub::Hash {
					hash_type: *hash_type,
				}
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
			UFONodeType::Hash { .. } => {
				if Into::<&str>::into(output_name) == "hash" {
					Some(0)
				} else {
					None
				}
			}
			_ => unreachable!(),
		}
	}
}
