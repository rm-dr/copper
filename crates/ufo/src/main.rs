use anyhow::Result;
use std::path::Path;
use ufo_storage::{api::Dataset, mem::MemDataset};
use ufo_util::data::PipelineDataType;

use ufo_pipeline::{
	input::{file::FileInput, PipelineInput, PipelineInputKind},
	output::{storage::StorageOutput, PipelineOutput, PipelineOutputKind},
	pipeline::Pipeline,
};

fn main() -> Result<()> {
	// Make dataset
	let mut dataset = {
		let mut d = MemDataset::new();
		let x = d.add_class("AudioFile").unwrap();
		d.add_attr(x, "album", PipelineDataType::Text).unwrap();
		d.add_attr(x, "artist", PipelineDataType::Text).unwrap();
		d.add_attr(x, "albm", PipelineDataType::Text).unwrap();
		d
	};
	println!("{:#?}", dataset);

	// Load pipeline
	let pipe = Pipeline::from_file(Path::new("pipeline.toml"))?;

	let input = match &pipe.get_config().input {
		PipelineInputKind::File => {
			let f = FileInput::new("data/freeze.flac".into());
			f.injest().unwrap()
		}
	};

	let o = pipe.run(input)?;

	match &pipe.get_config().output {
		PipelineOutputKind::DataSet { attrs } => {
			let c = dataset.get_class("AudioFile").unwrap();
			let mut e = StorageOutput::new(
				&mut dataset,
				c,
				attrs.iter().map(|(a, b)| (a.into(), *b)).collect(),
			);
			e.export(o.iter().map(|x| x.as_ref().map(|x| x.as_ref())).collect())?;
		}
	}

	println!("\n\n\n\n{:#?}", dataset);

	Ok(())
}
