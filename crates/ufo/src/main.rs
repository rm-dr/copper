use anyhow::Result;
use futures::executor::block_on;
use std::path::Path;
use ufo_pipeline::{
	input::{file::FileInput, PipelineInput, PipelineInputKind},
	output::{storage::StorageOutput, PipelineOutput, PipelineOutputKind},
	pipeline::Pipeline,
};
use ufo_storage::{
	api::Dataset,
	sea::{self, SeaDataset},
};
use ufo_util::data::PipelineDataType;

fn main() -> Result<()> {
	// Make dataset
	let mut dataset = {
		let mut d = SeaDataset::new("sqlite:./test.sqlite?mode=rwc", "ufo_db");
		block_on(d.connect())?;
		let x = block_on(d.add_class("AudioFile")).unwrap();
		block_on(d.add_attr(x, "album", PipelineDataType::Text)).unwrap();
		block_on(d.add_attr(x, "artist", PipelineDataType::Text)).unwrap();
		block_on(d.add_attr(x, "albm", PipelineDataType::Text)).unwrap();
		d
	};
	//println!("{:#?}", dataset);

	// Load pipeline
	let pipe = Pipeline::from_file(Path::new("pipeline.toml"))?;

	let input = match &pipe.get_config().input {
		PipelineInputKind::File => {
			let f = FileInput::new("data/freeze.flac".into());
			f.run().unwrap()
		}
	};

	let o = pipe.run(input)?;

	match &pipe.get_config().output {
		PipelineOutputKind::DataSet { attrs } => {
			let c = block_on(dataset.get_class("AudioFile")).unwrap();
			let mut e = StorageOutput::new(
				&mut dataset,
				c,
				attrs.iter().map(|(a, b)| (a.into(), *b)).collect(),
			);
			e.run(o.iter().map(|x| x.as_ref()).collect())?;
		}
	}

	//println!("\n\n\n\n{:#?}", dataset);

	if let Err(err) = block_on(sea::run()) {
		panic!("{}", err);
	}

	Ok(())
}
