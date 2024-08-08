use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use ufo_util::mime::MimeType;

#[derive(Deserialize, Serialize, Debug)]
pub struct UploadStartResult {
	pub handle: SmartString<LazyCompact>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UploadStartInfo {
	pub file_type: MimeType,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UploadNewFileResult {
	pub file_handle: SmartString<LazyCompact>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UploadFragmentMetadata {
	pub part_idx: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UploadFinish {
	pub hash: SmartString<LazyCompact>,
}
