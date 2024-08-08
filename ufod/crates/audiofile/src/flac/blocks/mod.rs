//! Read and write impelementations for all flac block types

// Not metadata blocks
mod header;
pub use header::{FlacMetablockHeader, FlacMetablockType};

mod audiodata;
pub use audiodata::FlacAudioFrame;

// Metadata blocks

mod streaminfo;
pub use streaminfo::FlacStreaminfoBlock;

mod picture;
pub use picture::FlacPictureBlock;

mod padding;
pub use padding::FlacPaddingBlock;

mod application;
pub use application::FlacApplicationBlock;

mod seektable;
pub use seektable::FlacSeektableBlock;

mod cuesheet;
pub use cuesheet::FlacCuesheetBlock;

mod comment;
pub use comment::FlacCommentBlock;

use super::errors::{FlacDecodeError, FlacEncodeError};
use std::io::Write;

/// A decode implementation for a
/// flac metadata block
pub trait FlacMetablockDecode: Sized {
	/// Try to decode this block from bytes.
	/// `data` should NOT include the metablock header.
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError>;
}

/// A encode implementation for a
/// flac metadata block
pub trait FlacMetablockEncode: Sized {
	/// Get the number of bytes that `encode()` will write.
	/// This does NOT include header length.
	fn get_len(&self) -> u32;

	/// Try to encode this block as bytes.
	fn encode(
		&self,
		is_last: bool,
		with_header: bool,
		target: &mut impl Write,
	) -> Result<(), FlacEncodeError>;
}
