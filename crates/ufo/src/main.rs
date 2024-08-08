use std::{fs::File, io::Read};

use anyhow::Result;

mod ingest;
mod storage;

use crate::ingest::{file::FileInjest, Ingest};

use ufo_pipeline::syntax::{prepareresult::PipelinePrepareResult, spec::PipelineSpec};

fn main() -> Result<()> {
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
	let o = p.run(f.injest().unwrap())?;
	println!("{:#?}\n\n", o);

	let f = FileInjest::new("data/top.mp3".into());
	let o = p.run(f.injest().unwrap())?;
	println!("{:#?}", o);

	Ok(())
}
