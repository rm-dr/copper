use std::{
	io::{Seek, SeekFrom},
	sync::Arc,
};
use ufo_audiofile::flac::flac_read_pictures;
use ufo_pipeline::api::{PipelineNode, PipelineNodeState};
use ufo_util::mime::MimeType;

use crate::{
	data::{UFOData, UFODataStub},
	errors::PipelineError,
	helpers::ArcVecBuffer,
	traits::UFOStaticNode,
	UFOContext,
};

pub struct ExtractCovers {
	mime: Option<MimeType>,
	fragments: ArcVecBuffer,
	is_done: bool,
}

impl ExtractCovers {
	pub fn new(_ctx: &<Self as PipelineNode>::NodeContext) -> Self {
		Self {
			mime: None,
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
						mime: format,
						fragment,
						is_last,
					} => (format, fragment, is_last),
					_ => unreachable!(),
				};

				assert!(!self.is_done);

				if let Some(f) = &self.mime {
					assert!(*f == format);
				} else {
					self.mime = Some(format);
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
		if self.mime.is_none() {
			return Ok(PipelineNodeState::Pending("args not ready"));
		}

		self.fragments.seek(SeekFrom::Start(0))?;
		let picture = match self.mime.as_ref().unwrap() {
			MimeType::Flac => {
				let pictures = flac_read_pictures(&mut self.fragments);
				if pictures.is_err() {
					return Ok(PipelineNodeState::Pending("malformed pictures"));
				}
				let mut pictures = pictures.unwrap();
				pictures.pop()
			}
			MimeType::Mp3 => unimplemented!(),
			_ => {
				return Err(PipelineError::UnsupportedDataType(format!(
					"cannot extract pictures from `{}`",
					self.mime.as_ref().unwrap()
				)))
			}
		};

		if let Some(picture) = picture {
			send_data(
				0,
				UFOData::Binary {
					mime: picture.get_mime().clone(),
					data: Arc::new(picture.take_img_data()),
				},
			)?;
		} else {
			send_data(0, UFOData::None(UFODataStub::Binary))?;
		}

		return Ok(PipelineNodeState::Done);
	}
}

impl UFOStaticNode for ExtractCovers {
	fn inputs() -> &'static [(&'static str, UFODataStub)] {
		&[("data", UFODataStub::Blob)]
	}

	fn outputs() -> &'static [(&'static str, UFODataStub)] {
		&[("cover_data", UFODataStub::Binary)]
	}
}
