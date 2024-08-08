use std::io::{Cursor, Read};

use crate::flac::errors::{FlacDecodeError, FlacEncodeError};

use super::{FlacMetablockDecode, FlacMetablockEncode, FlacMetablockHeader, FlacMetablockType};

/// A streaminfo block in a flac file
#[derive(Debug)]
pub struct FlacStreaminfoBlock {
	/// The minimum block size (in samples) used in the stream.
	pub min_block_size: u32,

	/// The maximum block size (in samples) used in the stream.
	/// (Minimum blocksize == maximum blocksize) implies a fixed-blocksize stream.
	pub max_block_size: u32,

	/// The minimum frame size (in bytes) used in the stream.
	/// May be 0 to imply the value is not known.
	pub min_frame_size: u32,

	/// The minimum frame size (in bytes) used in the stream.
	/// May be 0 to imply the value is not known.
	pub max_frame_size: u32,

	/// Sample rate in Hz. Though 20 bits are available,
	/// the maximum sample rate is limited by the structure of frame headers to 655350Hz.
	/// Also, a value of 0 is invalid.
	pub sample_rate: u32,

	/// (number of channels)-1. FLAC supports from 1 to 8 channels
	pub channels: u8,

	/// (bits per sample)-1. FLAC supports from 4 to 32 bits per sample.
	pub bits_per_sample: u8,

	/// Total samples in stream. 'Samples' means inter-channel sample, i.e. one second of 44.1Khz audio will have 44100 samples regardless of the number of channels. A value of zero here means the number of total samples is unknown.
	pub total_samples: u128,

	/// MD5 signature of the unencoded audio data. This allows the decoder to determine if an error exists in the audio data even when the error does not result in an invalid bitstream.
	pub md5_signature: [u8; 16],
}

impl FlacMetablockDecode for FlacStreaminfoBlock {
	fn decode(data: &[u8]) -> Result<Self, FlacDecodeError> {
		let mut d = Cursor::new(data);

		let min_block_size = {
			let mut block = [0u8; 4];
			d.read_exact(&mut block[2..])
				.map_err(|_| FlacDecodeError::MalformedBlock)?;
			u32::from_be_bytes(block)
		};

		let max_block_size = {
			let mut block = [0u8; 4];
			d.read_exact(&mut block[2..])
				.map_err(|_| FlacDecodeError::MalformedBlock)?;
			u32::from_be_bytes(block)
		};

		let min_frame_size = {
			let mut block = [0u8; 4];
			d.read_exact(&mut block[1..])
				.map_err(|_| FlacDecodeError::MalformedBlock)?;
			u32::from_be_bytes(block)
		};

		let max_frame_size = {
			let mut block = [0u8; 4];
			d.read_exact(&mut block[1..])
				.map_err(|_| FlacDecodeError::MalformedBlock)?;
			u32::from_be_bytes(block)
		};

		let (sample_rate, channels, bits_per_sample, total_samples) = {
			let mut block = [0u8; 8];
			d.read_exact(&mut block)
				.map_err(|_| FlacDecodeError::MalformedBlock)?;

			(
				// 20 bits: sample rate in hz
				u32::from_be_bytes([0, block[0], block[1], block[2]]) >> 4,
				// 3 bits: number of channels - 1.
				// FLAC supports 1 - 8 channels.
				((u8::from_le_bytes([block[2]]) & 0b0000_1110) >> 1) + 1,
				// 5 bits: bits per sample - 1.
				// FLAC supports 4 - 32 bps.
				((u8::from_le_bytes([block[2]]) & 0b0000_0001) << 4)
					+ ((u8::from_le_bytes([block[3]]) & 0b1111_0000) >> 4)
					+ 1,
				// 36 bits: total "cross-channel" samples in the stream.
				// (one second of 44.1Khz audio will have 44100 samples regardless of the number of channels)
				// Zero means we don't know.
				u128::from_be_bytes([
					0,
					0,
					0,
					0,
					//
					0,
					0,
					0,
					0,
					//
					0,
					0,
					0,
					block[3] & 0b0000_1111,
					//
					block[4],
					block[5],
					block[6],
					block[7],
				]),
			)
		};

		let md5_signature = {
			let mut block = [0u8; 16];
			d.read_exact(&mut block)
				.map_err(|_| FlacDecodeError::MalformedBlock)?;
			block
		};

		Ok(Self {
			min_block_size,
			max_block_size,
			min_frame_size,
			max_frame_size,
			sample_rate,
			channels,
			bits_per_sample,
			total_samples,
			md5_signature,
		})
	}
}

impl FlacMetablockEncode for FlacStreaminfoBlock {
	fn encode(
		&self,
		is_last: bool,
		target: &mut impl std::io::Write,
	) -> Result<(), FlacEncodeError> {
		let header = FlacMetablockHeader {
			block_type: FlacMetablockType::Streaminfo,
			length: 34,
			is_last,
		};

		header.encode(target)?;

		// TODO: enforce sizes

		target.write_all(&self.min_block_size.to_be_bytes()[2..])?;
		target.write_all(&self.max_block_size.to_be_bytes()[2..])?;
		target.write_all(&self.min_frame_size.to_be_bytes()[1..])?;
		target.write_all(&self.max_frame_size.to_be_bytes()[1..])?;

		// Layout of the next 8 bytes:
		// [8]: full bytes
		// [4 ]: first 4 bits are from this
		// [ 3]: next 3 bits are from this
		//
		// [8][8][4  ]: Sample rate
		// [ ][ ][ 3 ]: channels
		// [ ][ ][  1][4 ]: bits per sample
		// [ ][ ][   ][ 4][8 x 4]: total samples

		let mut out = [0u8; 8];

		let sample_rate = &self.sample_rate.to_be_bytes()[1..4];
		out[0] = (sample_rate[0] << 4) & 0b1111_0000;
		out[0] |= (sample_rate[1] >> 4) & 0b0000_1111;
		out[1] = (sample_rate[1] << 4) & 0b1111_0000;
		out[1] |= (sample_rate[2] >> 4) & 0b000_1111;
		out[2] = (sample_rate[2] << 4) & 0b1111_0000;

		let channels = self.channels - 1;
		out[2] |= (channels << 1) & 0b0000_1110;

		let bits_per_sample = self.bits_per_sample - 1;
		out[2] |= (bits_per_sample >> 4) & 0b0000_0001;
		out[3] |= (bits_per_sample << 4) & 0b1111_0000;

		let total_samples = self.total_samples.to_be_bytes();
		out[3] |= total_samples[10] & 0b0000_1111;
		out[4] = total_samples[12];
		out[5] = total_samples[13];
		out[6] = total_samples[14];
		out[7] = total_samples[15];

		target.write_all(&out)?;

		target.write_all(&self.md5_signature)?;

		return Ok(());
	}
}
