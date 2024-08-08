//! Read and write impelementations for all flac block types

// Not metadata blocks
mod header;
pub use header::{FlacMetablockHeader, FlacMetablockType};

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
	/// Try to encode this block as bytes.
	///
	/// Writes this block's data into `data`,
	/// including the metablock header.
	fn encode(&self, is_last: bool, target: &mut impl Write) -> Result<(), FlacEncodeError>;
}
