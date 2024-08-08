use std::sync::Arc;

use ufo_util::data::PipelineData;

use crate::{errors::PipelineError, PipelineStatelessRunner};

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
	fn run(
		&self,
		data: Vec<Option<Arc<PipelineData>>>,
	) -> Result<Vec<Option<Arc<PipelineData>>>, PipelineError> {
		let d = data.first().unwrap();
		let ifnone = data.get(1).unwrap();
		return Ok(vec![{
			match d {
				Some(x) => Some(x.clone()),
				None => ifnone.clone(),
			}
		}]);
	}
}
