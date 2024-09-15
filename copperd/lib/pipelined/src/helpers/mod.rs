mod s3client;

//
// MARK: Small helpers
//
pub use s3client::*;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::data::BytesStreamPacket;

pub enum OpenBytesSourceReader<'a> {
	Array(Arc<Vec<u8>>),
	Stream(broadcast::Receiver<BytesStreamPacket>),
	S3(S3Reader<'a>),
}
