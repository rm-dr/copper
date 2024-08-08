use std::{
	ffi::OsStr,
	fs::File,
	io::Read,
	path::{Path, PathBuf},
};

use ufo_pipeline::pipeline::pipeline::Pipeline;
use walkdir::WalkDir;

use super::api::Pipestore;

pub struct FsPipestore {
	pipe_storage_dir: PathBuf,
	pipeline_names: Vec<String>,
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

				pipeline_names.push(
					entry
						.path()
						.file_name()
						.unwrap()
						.to_str()
						.unwrap()
						.strip_suffix(".toml")
						.unwrap()
						.to_string(),
				)
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
		name: ufo_pipeline::labels::PipelineLabel,
		context: std::sync::Arc<<<ufo_pipeline_nodes::nodetype::UFONodeType as ufo_pipeline::api::PipelineNodeStub>::NodeType as ufo_pipeline::api::PipelineNode>::NodeContext>,
	) -> ufo_pipeline::pipeline::pipeline::Pipeline<ufo_pipeline_nodes::nodetype::UFONodeType> {
		let path_to_pipeline = self.pipe_storage_dir.join(format!("{name}.toml"));

		let mut f = File::open(path_to_pipeline).unwrap();
		let mut s = String::new();
		f.read_to_string(&mut s).unwrap();

		Pipeline::from_toml_str((&name).into(), &s, context).unwrap()
	}

	fn all_pipelines(&self) -> &Vec<String> {
		&self.pipeline_names
	}
}
