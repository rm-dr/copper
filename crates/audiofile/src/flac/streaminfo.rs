use std::io::Read;

use super::errors::FlacError;

// TODO: enforce flac constraints and write
pub struct FlacStreamInfo {
	min_block_size: u32,
	max_block_size: u32,
	min_frame_size: u32,
	max_frame_size: u32,
	sample_rate: u32,
	channels: u8,
	bits_per_sample: u8,
	total_samples: u128,
	md5_signature: [u8; 16],
}

impl FlacStreamInfo {
	pub fn decode<R>(mut read: R) -> Result<Self, FlacError>
	where
		R: Read,
	{
		// Use one buffer, since most reads are 4 bytes.
		// Be careful to reset this to zero!
		let mut block = [0u8; 4];

		let min_block_size = {
			read.read_exact(&mut block[2..])?;
			u32::from_be_bytes(block)
		};

		let max_block_size = {
			block = [0u8; 4];
			read.read_exact(&mut block[2..])?;
			u32::from_be_bytes(block)
		};

		let min_frame_size = {
			block = [0u8; 4];
			read.read_exact(&mut block[1..])?;
			u32::from_be_bytes(block)
		};

		let max_frame_size = {
			block = [0u8; 4];
			read.read_exact(&mut block[1..])?;
			u32::from_be_bytes(block)
		};

		let (sample_rate, channels, bits_per_sample, total_samples) = {
			let mut block = [0u8; 8];
			read.read_exact(&mut block)?;

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
			read.read_exact(&mut block)?;
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
