use crate::model::{ItemReader, ItemType};
use std::{collections::HashMap, error::Error, fmt::Display};

mod tags;
pub use tags::TagExtractor;

#[derive(Debug)]
pub enum ExtractorError {
	FileSystemError(Box<dyn Error>),
	UnsupportedDataType,
}

// TODO: clean up
impl Error for ExtractorError {}
unsafe impl Send for ExtractorError {}
unsafe impl Sync for ExtractorError {}
impl Display for ExtractorError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::FileSystemError(e) => write!(f, "Fs error: {e}"),
			Self::UnsupportedDataType => write!(f, "Unsupported Item data type"),
		}
	}
}

#[derive(Debug)]
pub enum ExtractorOutput {
	None,
	Text(String),
	Multi(HashMap<String, ExtractorOutput>),
}

pub trait Extractor {
	/// Does this extractor support items of type `x`?
	fn supports_type(data_type: ItemType) -> bool;

	/// Extract data from the given reader.
	/// Returns `None` if this data couldn't be found.
	fn extract(
		data_type: ItemType,
		data_read: &mut dyn ItemReader,
	) -> Result<ExtractorOutput, ExtractorError>;
}
