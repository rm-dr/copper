use std::fmt::Debug;

use crate::{
	common::vorbiscomment::VorbisComment,
	flac::errors::{FlacDecodeError, FlacEncodeError},
};

use super::{FlacMetablockDecode, FlacMetablockEncode, FlacMetablockHeader, FlacMetablockType};

/// A vorbis comment metablock in a flac file
pub struct FlacCommentBlock {
	/// The vorbis comment stored inside this block
	pub comment: VorbisComment,
}

impl Debug for FlacCommentBlock {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("FlacCommentBlock")
			.field("comment", &self.comment)
			.finish()
	}
}

impl FlacMetablockDecode for FlacCommentBlock {
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		let comment = VorbisComment::decode(data)?;
		Ok(Self { comment })
	}
}

impl FlacMetablockEncode for FlacCommentBlock {
	fn get_len(&self) -> u32 {
		self.comment.get_len()
	}

	fn encode(
		&self,
		is_last: bool,
		with_header: bool,
		target: &mut impl std::io::Write,
	) -> Result<(), FlacEncodeError> {
		if with_header {
			let header = FlacMetablockHeader {
				block_type: FlacMetablockType::VorbisComment,
				length: self.get_len(),
				is_last,
			};
			header.encode(target)?;
		}

		self.comment.encode(target)?;
		return Ok(());
	}
}
