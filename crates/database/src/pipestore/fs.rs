use std::{
	ffi::OsStr,
	fs::File,
	io::Read,
	path::{Path, PathBuf},
};

use walkdir::WalkDir;

use super::api::Pipestore;

pub struct FsPipestore {
	pipe_storage_dir: PathBuf,
	pipeline_names: Vec<String>,
}

// TODO: remove all quick send+sync and properly handle parallism
unsafe impl Send for FsPipestore {}

impl FsPipestore {
	pub(crate) fn create(pipe_storage_dir: &Path) -> Result<(), ()> {
		if pipe_storage_dir.exists() {
			return Err(());
		}

		std::fs::create_dir(pipe_storage_dir).unwrap();
		Ok(())
	}

	pub(crate) fn open(pipe_storage_dir: &Path) -> Result<Self, ()> {
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
	fn load_pipeline(&self, name: ufo_pipeline::labels::PipelineLabel) -> String {
		let path_to_pipeline = self.pipe_storage_dir.join(format!("{name}.toml"));

		let mut f = File::open(path_to_pipeline).unwrap();
		let mut s = String::new();
		f.read_to_string(&mut s).unwrap();
		return s;
	}

	fn all_pipelines(&self) -> &[String] {
		&self.pipeline_names
	}
}

/*
impl<NodeStub: PipelineNodeStub> Pipestore<NodeStub> for FsPipestore<NodeStub> {
	fn load_pipeline(
		&self,
		name: ufo_pipeline::labels::PipelineLabel,
		context: Arc<<NodeStub::NodeType as PipelineNode>::NodeContext>,
	) -> Pipeline<NodeStub> {
		let path_to_pipeline = self.pipe_storage_dir.join(format!("{name}.toml"));

		let mut f = File::open(path_to_pipeline).unwrap();
		let mut s = String::new();
		f.read_to_string(&mut s).unwrap();

		Pipeline::from_toml_str((&name).into(), &s, context).unwrap()
	}
}
*/
