use std::sync::Arc;

use crate::{data::PipelineData, errors::PipelineError, PipelineStatelessRunner};

pub struct IfNone {}

impl IfNone {
	pub fn new() -> Self {
		Self {}
	}
}

impl PipelineStatelessRunner for IfNone {
	fn run(
		&self,
		data_packet: Vec<Option<Arc<PipelineData>>>,
	) -> Result<Vec<Option<Arc<PipelineData>>>, PipelineError> {
		let data = data_packet.first().unwrap();
		let ifnone = data_packet.get(1).unwrap();
		return Ok(vec![{
			match data {
				Some(x) => Some(x.clone()),
				None => ifnone.clone(),
			}
		}]);
	}
}
