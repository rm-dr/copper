use std::{
	io::{Seek, SeekFrom},
	sync::Arc,
};
use ufo_audiofile::flac::flac_read_pictures;
use ufo_metadb::data::MetaDbDataStub;
use ufo_pipeline::api::{PipelineNode, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{
	data::UFOData, errors::PipelineError, helpers::ArcVecBuffer, traits::UFOStaticNode, UFOContext,
};

pub struct ExtractCovers {
	format: Option<MimeType>,
	fragments: ArcVecBuffer,
	is_done: bool,
}

impl ExtractCovers {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			format: None,
			fragments: ArcVecBuffer::new(),
			is_done: false,
		}
	}
}

impl PipelineNode for ExtractCovers {
	type NodeContext = UFOContext;
	type DataType = UFOData;
	type ErrorType = PipelineError;

	fn take_input(&mut self, (port, data): (usize, UFOData)) -> Result<(), PipelineError> {
		match port {
			0 => {
				let (format, fragment, is_last) = match data {
					UFOData::Blob {
						format,
						fragment,
						is_last,
					} => (format, fragment, is_last),
					_ => panic!(),
				};

				assert!(!self.is_done);

				if let Some(f) = &self.format {
					assert!(*f == format);
				} else {
					self.format = Some(format);
				}

				self.fragments.push_back(fragment);
				self.is_done = is_last;
			}
			_ => unreachable!(),
		}
		return Ok(());
	}

	fn run<F>(&mut self, send_data: F) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, Self::DataType) -> Result<(), PipelineError>,
	{
		if self.format.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		self.fragments.seek(SeekFrom::Start(0))?;
		let picture = match self.format.as_ref().unwrap() {
			MimeType::Flac => {
				let pictures = flac_read_pictures(&mut self.fragments);
				if pictures.is_err() {
					return Ok(PipelineNodeState::Pending("malformed pictures"));
				}
				let mut pictures = pictures.unwrap();
				pictures.pop()
			}
			MimeType::Mp3 => unimplemented!(),
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		if let Some(picture) = picture {
			send_data(
				0,
				UFOData::Binary {
					format: picture.get_mime().clone(),
					data: Arc::new(picture.take_img_data()),
				},
			)?;
		} else {
			send_data(0, UFOData::None(MetaDbDataStub::Binary))?;
		}

		return Ok(PipelineNodeState::Done);
	}
}

impl UFOStaticNode for ExtractCovers {
	fn inputs() -> &'static [(&'static str, ufo_metadb::data::MetaDbDataStub)] {
		&[("data", MetaDbDataStub::Blob)]
	}

	fn outputs() -> &'static [(&'static str, MetaDbDataStub)] {
		&[("cover_data", MetaDbDataStub::Binary)]
	}
}
