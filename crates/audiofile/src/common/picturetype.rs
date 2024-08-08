use std::fmt::Display;

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

/// A picture type according to the ID3v2 APIC frame
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
}
