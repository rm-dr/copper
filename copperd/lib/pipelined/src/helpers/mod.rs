mod s3client;
use std::sync::Arc;

use tokio::sync::broadcast;

//
// MARK: Small helpers
//
pub use s3client::*;

pub enum OpenBytesSourceReader<'a> {
	Array(broadcast::Receiver<Arc<Vec<u8>>>),
	S3(S3Reader<'a>),
}
