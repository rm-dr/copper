use anyhow::Result;
use futures::executor::block_on;
use std::path::Path;
use ufo_pipeline::{
	input::{file::FileInput, PipelineInput, PipelineInputKind},
	output::{storage::StorageOutput, PipelineOutput, PipelineOutputKind},
	pipeline::Pipeline,
};
use ufo_storage::{
	api::{AttributeOptions, Dataset},
	sea::dataset::SeaDataset,
};
use ufo_util::data::PipelineDataType;

fn main() -> Result<()> {
	// Make dataset
	let mut dataset = {
		let mut d = SeaDataset::new("sqlite:./test.sqlite?mode=rwc", "ufo_db");
		block_on(d.connect())?;
		let x = block_on(d.add_class("AudioFile")).unwrap();
		block_on(d.add_attr(x, "album", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(x, "artist", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(
			x,
			"albumartist",
			PipelineDataType::Text,
			AttributeOptions::new(),
		))
		.unwrap();
		block_on(d.add_attr(
			x,
			"tracknumber",
			PipelineDataType::Text,
			AttributeOptions::new(),
		))
		.unwrap();
		block_on(d.add_attr(x, "year", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(x, "genre", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(x, "ISRC", PipelineDataType::Text, AttributeOptions::new())).unwrap();
		block_on(d.add_attr(x, "lyrics", PipelineDataType::Text, AttributeOptions::new())).unwrap();

		block_on(d.add_attr(
			x,
			"audio_data",
			PipelineDataType::Binary,
			AttributeOptions::new(),
		))
		.unwrap();

		let x = block_on(d.add_class("CoverArt")).unwrap();
		block_on(d.add_attr(
			x,
			"image_data",
			PipelineDataType::Binary,
			AttributeOptions::new(),
		))
		.unwrap();
		block_on(d.add_attr(
			x,
			"content_hash",
			PipelineDataType::Text,
			AttributeOptions::new().unique(true),
		))
		.unwrap();
		d
	};

	// Load pipeline
	let pipe = Pipeline::from_file(Path::new("pipeline.toml"))?;

	for p in ["data/freeze.flac", "data/png.flac"] {
		let input = match &pipe.get_config().input {
			PipelineInputKind::File => {
				let f = FileInput::new(p.into());
				f.run().unwrap()
			}
		};

		let o = pipe.run(4, input)?;

		match &pipe.get_config().output {
			PipelineOutputKind::DataSet { attrs } => {
				let c = block_on(dataset.get_class("AudioFile"))?.unwrap();
				let mut e = StorageOutput::new(
					&mut dataset,
					c,
					attrs.iter().map(|(a, b)| (a.into(), *b)).collect(),
				);
				e.run(o.iter().collect())?;
			}
		}
	}

	Ok(())
}
