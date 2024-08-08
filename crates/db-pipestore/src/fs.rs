use std::{
	ffi::OsStr,
	fs::File,
	io::Read,
	path::{Path, PathBuf},
	sync::Arc,
};

use ufo_pipeline::{
	api::{PipelineNode, PipelineNodeStub},
	labels::PipelineName,
	pipeline::pipeline::Pipeline,
};
use ufo_pipeline_nodes::nodetype::UFONodeType;
use walkdir::WalkDir;

use super::api::Pipestore;

pub struct FsPipestore {
	pipe_storage_dir: PathBuf,
	pipeline_names: Vec<PipelineName>,
}

// TODO: remove all quick send+sync and properly handle parallism
unsafe impl Send for FsPipestore {}

impl FsPipestore {
	pub fn create(pipe_storage_dir: &Path) -> Result<(), ()> {
		if pipe_storage_dir.exists() {
			return Err(());
		}

		std::fs::create_dir(pipe_storage_dir).unwrap();
		Ok(())
	}

	pub fn open(pipe_storage_dir: &Path) -> Result<Self, ()> {
		let mut pipeline_names = Vec::new();
		for entry in WalkDir::new(pipe_storage_dir) {
			let entry = entry.unwrap();
			if entry.path().is_file() {
				if entry.path().extension() != Some(OsStr::new("toml")) {
					panic!()
				}

				pipeline_names.push(PipelineName::new(
					entry
						.path()
						.file_name()
						.unwrap()
						.to_str()
						.unwrap()
						.strip_suffix(".toml")
						.unwrap(),
				))
			}
		}

		Ok(Self {
			pipe_storage_dir: pipe_storage_dir.to_path_buf(),
			pipeline_names,
		})
	}
}

impl Pipestore for FsPipestore {
	fn load_pipeline(
		&self,
		name: &PipelineName,
		context: Arc<<<UFONodeType as PipelineNodeStub>::NodeType as PipelineNode>::NodeContext>,
	) -> Option<Pipeline<UFONodeType>> {
		let path_to_pipeline = self.pipe_storage_dir.join(format!("{name}.toml"));

		if !path_to_pipeline.is_file() {
			return None;
		}

		let mut f = File::open(path_to_pipeline).unwrap();
		let mut s = String::new();
		f.read_to_string(&mut s).unwrap();

		Some(Pipeline::from_toml_str(name.name().as_str(), &s, context).unwrap())
	}

	fn all_pipelines(&self) -> &Vec<PipelineName> {
		&self.pipeline_names
	}
}
