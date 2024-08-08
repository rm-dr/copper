use crate::flac::errors::{FlacDecodeError, FlacEncodeError};

/// An audio frame in a flac file
pub struct FlacAudioFrame {
	/// The audio frame
	pub data: Vec<u8>,
}

impl FlacAudioFrame {
	/// Decode the given data as a flac audio frame.
	/// This should start with a sync sequence.
	pub fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		if data.len() <= 2 {
			return Err(FlacDecodeError::MalformedBlock);
		}

		if !(data[0] == 0b1111_1111 && data[1] & 0b1111_1100 == 0b1111_1000) {
			return Err(FlacDecodeError::BadSyncBytes);
		}

		// TODO: look for sync bytes inside frame
		// (There shouldn't be any)

		Ok(Self {
			data: Vec::from(data),
		})
	}
}

impl FlacAudioFrame {
	/// Encode this audio frame.
	pub fn encode(&self, target: &mut impl std::io::Write) -> Result<(), FlacEncodeError> {
		target.write_all(&self.data)?;
		return Ok(());
	}
}
