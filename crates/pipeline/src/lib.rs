//#![warn(missing_docs)]

mod graph;
#[allow(clippy::module_inception)]
mod pipeline;
mod syntax;

pub mod api;
pub mod errors;
pub mod labels;
pub mod portspec;
pub mod runner;

use api::{PipelineData, PipelineNode, PipelineNodeStub};

// Shortcut types
pub type SNodeType<StubType> = <StubType as PipelineNodeStub>::NodeType;
pub type SDataType<StubType> = <<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType;
pub type SDataStub<StubType> = <<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub;
pub type NDataStub<NodeType> = <<NodeType as PipelineNode>::DataType as PipelineData>::DataStub;
