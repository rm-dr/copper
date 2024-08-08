use std::sync::Arc;
use ufo_util::data::PipelineData;

use crate::{errors::PipelineError, PipelineStatelessRunner};

#[derive(Clone)]
pub struct IfNone {}

impl IfNone {
	pub fn new() -> Self {
		Self {}
	}
}

impl Default for IfNone {
	fn default() -> Self {
		Self::new()
	}
}

impl PipelineStatelessRunner for IfNone {
	fn run(&self, data: Vec<Arc<PipelineData>>) -> Result<Vec<Arc<PipelineData>>, PipelineError> {
		let d = data.first().unwrap();
		let ifnone = data.get(1).unwrap();
		return Ok(vec![{
			match *d.as_ref() {
				PipelineData::None(_) => ifnone.clone(),
				_ => d.clone(),
			}
		}]);
	}
}
