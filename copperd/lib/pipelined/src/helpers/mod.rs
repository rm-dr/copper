mod s3client;

//
// MARK: Small helpers
//
pub use s3client::*;
use tokio::sync::broadcast;

use crate::data::BytesStreamPacket;

pub enum OpenBytesSourceReader<'a> {
	Array(broadcast::Receiver<BytesStreamPacket>),
	S3(S3Reader<'a>),
}
