mod checkresult;
mod labels;
mod pipeline;
mod ports;

pub use checkresult::PipelineCheckResult;
pub use labels::{PipelineNodeLabel, PipelinePortLabel};
pub use pipeline::{Pipeline, PipelineConfig, PipelineNodeSpec};
pub use ports::{PipelineInput, PipelineOutput};

// TODO: enforce docs
// TODO: node id, port id type
