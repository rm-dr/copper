use std::{fs::File, io::Read};

use anyhow::Result;

mod ingest;
mod model;
mod storage;

use storage::StorageBackend;

use crate::ingest::{file::FileInjest, Ingest};

use ufo_pipeline::syntax::{prepareresult::PipelinePrepareResult, spec::PipelineSpec};

fn main() -> Result<()> {
	let mut x = storage::MemStorageBackend::new();

	let d = x.add_class("Class").unwrap();
	x.add_attr(d, "test attr", model::AttributeType::String);

	// Load pipeline
	let mut f = File::open("pipeline.toml").unwrap();
	let mut s: String = Default::default();
	f.read_to_string(&mut s)?;
	let mut p: PipelineSpec = toml::from_str(&s)?;
	let p = p.prepare();
	let p = match p {
		PipelinePrepareResult::Ok(x) => x,
		_ => panic!(),
	};

	// Run pipeline
	let f = FileInjest::new("data/freeze.flac".into());
	println!("{:#?}\n\n", p.run(f.injest().unwrap())?);

	let f = FileInjest::new("data/top.mp3".into());
	println!("{:#?}", p.run(f.injest().unwrap())?);

	Ok(())
}
