mod bytessourcereader;
pub use bytessourcereader::*;

mod connectedinput;
pub use connectedinput::*;

mod s3reader;
pub use s3reader::*;

//
// MARK: Small helpers
//

pub enum OpenBytesSourceReader {
	Array(BytesSourceArrayReader),
	S3(S3Reader),
}
