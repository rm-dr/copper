use std::{fs::File, io::Read};

use anyhow::Result;

mod ingest;
mod storage;

use crate::ingest::{file::FileInjest, Ingest};

use ufo_pipeline::syntax::spec::{PipelineInput, PipelineOutput, PipelineSpec};

fn main() -> Result<()> {
	// Load pipeline
	let mut f = File::open("pipeline.toml").unwrap();
	let mut s: String = Default::default();
	f.read_to_string(&mut s)?;
	let spec: PipelineSpec = toml::from_str(&s)?;
	let pipe = match spec.prepare() {
		Ok(x) => x,
		Err(x) => {
			println!("{:?}", x);
			panic!()
		}
	};

	let input = match &pipe.get_config().input {
		PipelineInput::File => {
			let f = FileInjest::new("data/freeze.flac".into());
			f.injest().unwrap()
		}
	};

	let o = pipe.run(input)?;

	match &pipe.get_config().output {
		PipelineOutput::DataSet { class } => {
			println!("{class}: {:?}", o);
		}
	}

	Ok(())
}
