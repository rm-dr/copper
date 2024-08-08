use crate::{
	data::PipelineData,
	errors::PipelineError,
	nodes::{PipelineNode, PipelineNodeState},
};

#[derive(Clone)]
pub struct Constant {
	value: PipelineData,
}

impl Constant {
	pub fn new(value: PipelineData) -> Self {
		Self { value }
	}
}

impl PipelineNode for Constant {
	fn init<F>(
		&mut self,
		send_data: F,
		input: Vec<PipelineData>,
	) -> Result<PipelineNodeState, PipelineError>
	where
		F: Fn(usize, PipelineData) -> Result<(), PipelineError>,
	{
		assert!(input.len() == 0);
		send_data(0, self.value.clone())?;
		Ok(PipelineNodeState::Done)
	}
}
