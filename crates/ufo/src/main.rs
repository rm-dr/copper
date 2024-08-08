use anyhow::Result;
use std::{fs::File, io::Read};
use ufo_storage::{api::Dataset, mem::MemDataset};
use ufo_util::data::PipelineDataType;

use ufo_pipeline::{
	input::{file::FileInput, PipelineInput, PipelineInputKind},
	output::{storage::StorageOutput, PipelineOutput, PipelineOutputKind},
	syntax::spec::PipelineSpec,
};

fn main() -> Result<()> {
	// Make dataset
	let mut dataset = {
		let mut d = MemDataset::new();
		let x = d.add_class("AudioFile").unwrap();
		d.add_attr(x, "a", PipelineDataType::Text).unwrap();
		d.add_attr(x, "b", PipelineDataType::Text).unwrap();
		d
	};
	println!("{:#?}", dataset);

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
		PipelineInputKind::File => {
			let f = FileInput::new("data/freeze.flac".into());
			f.injest().unwrap()
		}
	};

	let o = pipe.run(input)?;

	match &pipe.get_config().output {
		PipelineOutputKind::DataSet { class_name } => {
			let c = dataset.get_class(&class_name).unwrap();
			let mut e = StorageOutput::new(&mut dataset, c);
			e.export(o.iter().map(|x| x.as_ref().map(|x| x.as_ref())).collect())?;
		}
	}

	println!("\n\n\n\n{:#?}", dataset);

	Ok(())
}
