use std::{
	collections::HashMap,
	fs::File,
	io::{Read, Seek},
};

use anyhow::Result;

mod model;
mod pipeline;
mod storage;

use model::ItemReader;
use storage::StorageBackend;

use crate::{
	model::{AudioItemType, ItemType},
	pipeline::{
		components::Pipeline,
		data::{AudioFormat, BinaryFormat, PipelineData},
		nodes::PipelineNode,
	},
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

	// Load pipeline
	let mut f = File::open("pipeline.toml").unwrap();
	let mut s: String = Default::default();
	f.read_to_string(&mut s)?;
	let mut p: Pipeline = toml::from_str(&s)?;
	println!("{:?}", p.check());

	// Run pipeline
	let f = File::open("data/freeze.flac").unwrap();
	let mut f = FileReader(f);
	x.add_item(d, ItemType::Audio(AudioItemType::Flac), &mut f)
		.unwrap();

	f.rewind()?;

	let mut data = Vec::new();
	f.read_to_end(&mut data)?;

	println!(
		"{:#?}",
		p.run(HashMap::from([(
			"data".into(),
			Some(PipelineData::Binary {
				format: BinaryFormat::Audio(AudioFormat::Flac),
				data
			})
		)]))?
	);

	let f = File::open("data/top.mp3").unwrap();
	let mut f = FileReader(f);
	x.add_item(d, ItemType::Audio(AudioItemType::Mp3), &mut f)
		.unwrap();

	f.rewind()?;
	let mut data = Vec::new();
	f.read_to_end(&mut data)?;

	println!(
		"{:#?}",
		p.run(HashMap::from([(
			"data".into(),
			Some(PipelineData::Binary {
				format: BinaryFormat::Audio(AudioFormat::Mp3),
				data
			})
		)]))?
	);

	Ok(())
}
