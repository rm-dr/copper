use std::{
	fs::File,
	io::{Read, Seek},
};

use anyhow::Result;

mod extract;
mod model;
mod storage;

use model::ItemReader;
use storage::StorageBackend;

use crate::{
	extract::Extractor,
	model::{AudioItemType, ItemType},
};

struct FileReader(File);

impl Read for FileReader {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.0.read(buf)
	}
}

impl Seek for FileReader {
	fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
		self.0.seek(pos)
	}
}

impl ItemReader<'_> for FileReader {}

fn main() -> Result<()> {
	let mut x = storage::MemStorageBackend::new();

	let d = x.add_class("Class").unwrap();
	x.add_attr(d, "test attr", model::AttributeType::String);

	let f = File::open("data/freeze.flac").unwrap();
	let mut f = FileReader(f);
	x.add_item(d, ItemType::Audio(AudioItemType::Flac), &mut f)
		.unwrap();

	f.rewind()?;
	println!(
		"{:#?}",
		extract::TagExtractor::extract(ItemType::Audio(AudioItemType::Flac), &mut f)?
	);

	let f = File::open("data/top.mp3").unwrap();
	let mut f = FileReader(f);
	let i = x
		.add_item(d, ItemType::Audio(AudioItemType::Mp3), &mut f)
		.unwrap();

	f.rewind()?;
	println!(
		"{:#?}",
		extract::TagExtractor::extract(ItemType::Audio(AudioItemType::Mp3), &mut f)?
	);

	println!("{:?}", x.get_item(i).unwrap());

	Ok(())
}
