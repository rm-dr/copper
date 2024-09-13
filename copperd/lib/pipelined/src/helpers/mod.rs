mod s3reader;
use std::sync::Arc;

use tokio::sync::broadcast;

//
// MARK: Small helpers
//
pub use s3reader::*;

pub enum OpenBytesSourceReader {
	Array(broadcast::Receiver<Arc<Vec<u8>>>),
	S3(S3Reader),
}
