use crossbeam::channel::Receiver;
use itertools::Itertools;
use std::{io::Seek, sync::Arc};
use ufo_audiofile::{common::tagtype::TagType, flac::flac_read_tags};
use ufo_metadb::data::{MetaDbData, MetaDbDataStub};
use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeState},
	labels::PipelinePortLabel,
};
use ufo_util::mime::MimeType;

use crate::{
	errors::PipelineError, helpers::ArcVecBuffer, nodetype::UFONodeType, traits::UFONode,
	UFOContext,
};

// TODO: fail after max buffer size
pub struct ExtractTags {
	data: Option<MetaDbData>,
	tags: Vec<TagType>,
	buffer: ArcVecBuffer,
	input_receiver: Receiver<(usize, MetaDbData)>,
}

impl ExtractTags {
	pub fn new(
		_ctx: &<Self as PipelineNode>::NodeContext,
		input_receiver: Receiver<(usize, MetaDbData)>,
		tags: Vec<TagType>,
	) -> Self {
		Self {
			data: None,
			tags: tags.into_iter().unique().collect(),
			buffer: ArcVecBuffer::new(),
			input_receiver,
		}
	}
}

impl PipelineNode for ExtractTags {
	type NodeContext = UFOContext;
	type DataType = MetaDbData;
	type ErrorType = PipelineError;

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
					_ => unreachable!(),
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
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		let (data_type, data) = match self.data.as_mut().unwrap() {
			MetaDbData::Blob {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!(),
		};

		let (changed, done) = self.buffer.recv_all(data);
		match (changed, done) {
			(false, true) => unreachable!(),
			(false, false) => return Ok(PipelineNodeState::Pending("no new data")),
			(true, true) | (true, false) => {}
		}

		self.buffer.seek(std::io::SeekFrom::Start(0)).unwrap();
		let tagger = match data_type {
			MimeType::Flac => {
				let r = flac_read_tags(&mut self.buffer);
				if r.is_err() {
					return Ok(PipelineNodeState::Pending("malformed block"));
				}
				r.unwrap()
			}
			MimeType::Mp3 => unimplemented!(),
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		if tagger.is_none() {
			return Ok(PipelineNodeState::Pending("no comment block found"));
		}
		let tagger = tagger.unwrap();

		for (i, tag_type) in self.tags.iter().enumerate() {
			if let Some(tag_value) = tagger.get_tag(tag_type) {
				send_data(i, MetaDbData::Text(Arc::new(tag_value)))?;
			} else {
				send_data(i, MetaDbData::None(MetaDbDataStub::Text))?;
			}
		}

		return Ok(PipelineNodeState::Done);
	}
}

impl ExtractTags {
	fn inputs() -> &'static [(&'static str, MetaDbDataStub)] {
		&[("data", MetaDbDataStub::Blob)]
	}
}

impl UFONode for ExtractTags {
	fn n_inputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::ExtractTags { .. } => Self::inputs().len(),
			_ => unreachable!(),
		}
	}

	fn input_compatible_with(
		stub: &UFONodeType,
		ctx: &UFOContext,
		input_idx: usize,
		input_type: MetaDbDataStub,
	) -> bool {
		Self::input_default_type(stub, ctx, input_idx) == input_type
	}

	fn input_with_name(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_name: &PipelinePortLabel,
	) -> Option<usize> {
		match stub {
			UFONodeType::ExtractTags { .. } => Self::inputs()
				.iter()
				.enumerate()
				.find(|(_, (n, _))| PipelinePortLabel::from(*n) == *input_name)
				.map(|(x, _)| x),
			_ => unreachable!(),
		}
	}

	fn input_default_type(
		stub: &UFONodeType,
		_ctx: &UFOContext,
		input_idx: usize,
	) -> MetaDbDataStub {
		match stub {
			UFONodeType::ExtractTags { .. } => Self::inputs().get(input_idx).unwrap().1,
			_ => unreachable!(),
		}
	}

	fn n_outputs(stub: &UFONodeType, _ctx: &UFOContext) -> usize {
		match stub {
			UFONodeType::ExtractTags { tags } => tags.len(),
			_ => unreachable!(),
		}
	}

	fn output_type(stub: &UFONodeType, _ctx: &UFOContext, output_idx: usize) -> MetaDbDataStub {
		match stub {
			UFONodeType::ExtractTags { tags } => {
				assert!(output_idx < tags.len());
				MetaDbDataStub::Text
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
			UFONodeType::ExtractTags { tags } => tags
				.iter()
				.enumerate()
				.find(|(_, t)| PipelinePortLabel::from(Into::<&str>::into(*t)) == *output_name)
				.map(|(x, _)| x),
			_ => unreachable!(),
		}
	}
}
