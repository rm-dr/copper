//! An audio picture type, according to the ID3v2 APIC frame

use std::fmt::Display;

/// We failed to decode a picture type
#[derive(Debug)]
pub struct PictureTypeError {
	idx: u32,
}

impl Display for PictureTypeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Bad picture type `{}`", self.idx)
	}
}

impl std::error::Error for PictureTypeError {}

// TODO: There may only be one each of picture type 1 and 2 in a file.
// TODO: The MIME type may also be --> to signify that the data part is a URL of the picture instead of the picture data itself.

/// A picture type according to the ID3v2 APIC frame
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq)]
pub enum PictureType {
	Other,
	PngFileIcon,
	OtherFileIcon,
	FrontCover,
	BackCover,
	LeafletPage,
	Media,
	LeadArtist,
	Artist,
	Conductor,
	BandOrchestra,
	Composer,
	Lyricist,
	RecLocation,
	DuringRecording,
	DuringPerformance,
	VideoScreenCapture,
	ABrightColoredFish,
	Illustration,
	ArtistLogotype,
	PublisherLogotype,
}

impl PictureType {
	/// Try to decode a picture type from the given integer.
	/// Returns an error if `idx` is invalid.
	pub fn from_idx(idx: u32) -> Result<Self, PictureTypeError> {
		Ok(match idx {
			0 => PictureType::Other,
			1 => PictureType::PngFileIcon,
			2 => PictureType::OtherFileIcon,
			3 => PictureType::FrontCover,
			4 => PictureType::BackCover,
			5 => PictureType::LeafletPage,
			6 => PictureType::Media,
			7 => PictureType::LeadArtist,
			8 => PictureType::Artist,
			9 => PictureType::Conductor,
			10 => PictureType::BandOrchestra,
			11 => PictureType::Composer,
			12 => PictureType::Lyricist,
			13 => PictureType::RecLocation,
			14 => PictureType::DuringRecording,
			15 => PictureType::DuringPerformance,
			16 => PictureType::VideoScreenCapture,
			17 => PictureType::ABrightColoredFish,
			18 => PictureType::Illustration,
			19 => PictureType::ArtistLogotype,
			20 => PictureType::PublisherLogotype,
			_ => return Err(PictureTypeError { idx }),
		})
	}

	/// Return the index of this picture type
	pub fn to_idx(&self) -> u32 {
		match self {
			PictureType::Other => 0,
			PictureType::PngFileIcon => 1,
			PictureType::OtherFileIcon => 2,
			PictureType::FrontCover => 3,
			PictureType::BackCover => 4,
			PictureType::LeafletPage => 5,
			PictureType::Media => 6,
			PictureType::LeadArtist => 7,
			PictureType::Artist => 8,
			PictureType::Conductor => 9,
			PictureType::BandOrchestra => 10,
			PictureType::Composer => 11,
			PictureType::Lyricist => 12,
			PictureType::RecLocation => 13,
			PictureType::DuringRecording => 14,
			PictureType::DuringPerformance => 15,
			PictureType::VideoScreenCapture => 16,
			PictureType::ABrightColoredFish => 17,
			PictureType::Illustration => 18,
			PictureType::ArtistLogotype => 19,
			PictureType::PublisherLogotype => 20,
		}
	}
}
