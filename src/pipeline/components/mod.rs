mod checkresult;
mod labels;
mod pipeline;
mod ports;

pub use checkresult::PipelineCheckResult;
pub use labels::{PipelineNode, PipelinePort};
pub use pipeline::{Pipeline, PipelineConfig, PipelineNodeSpec};
pub use ports::{NodeInput, NodeOutput};

// TODO: enforce docs
