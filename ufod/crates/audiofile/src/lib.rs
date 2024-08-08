#![warn(missing_docs)]

//! Read and write audio file metadata.

pub mod common;
pub mod flac;

/// A decode implementation for a data block
/// in an arbitrary file format
pub trait FileBlockDecode: Sized {
	/// Errors we can encounter while decoding this block
	type DecodeErrorType: std::error::Error;

	/// Try to decode this block from bytes
	fn decode(data: &[u8]) -> Result<Self, Self::DecodeErrorType>;
}

/// An encode implementation for a data block
/// in an arbitrary file format
pub trait FileBlockEncode: Sized {
	/// Errors we can encounter while encoding this block
	type EncodeErrorType: std::error::Error;

	/// Try to encode this block as bytes
	fn encode(&self) -> Result<Vec<u8>, Self::EncodeErrorType>;
}
