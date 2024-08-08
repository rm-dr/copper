use std::{
	io::{Cursor, Read},
	sync::Arc,
};
use ufo_audiofile::flac::metastrip::FlacMetaStrip;
use ufo_util::data::{AudioFormat, BinaryFormat, PipelineData};

use crate::{errors::PipelineError, PipelineStatelessRunner};

pub struct StripTags {}

impl StripTags {
	pub fn new() -> Self {
		Self {}
	}
}

impl PipelineStatelessRunner for StripTags {
	fn run(&self, data: Vec<Arc<PipelineData>>) -> Result<Vec<Arc<PipelineData>>, PipelineError> {
		let data = data.first().unwrap();

		let (data_type, data) = match data.as_ref() {
			PipelineData::Binary {
				format: data_type,
				data,
			} => (data_type, data),
			_ => panic!(),
		};

		// TODO: stream data
		let data_read = Cursor::new(data);
		let stripped = match data_type {
			BinaryFormat::Audio(x) => match x {
				AudioFormat::Flac => {
					let mut x = FlacMetaStrip::new(data_read).unwrap();
					let mut v = Vec::new();
					x.read_to_end(&mut v).unwrap();
					v
				}
				AudioFormat::Mp3 => unimplemented!(),
			},
			_ => return Err(PipelineError::UnsupportedDataType),
		};

		let mut out = Vec::with_capacity(1);
		out.push(Arc::new(PipelineData::Binary {
			format: BinaryFormat::Audio(AudioFormat::Flac),
			data: stripped,
		}));

		return Ok(out);
	}
}
