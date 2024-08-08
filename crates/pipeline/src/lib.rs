//! Fast, flexible, parallel data processing pipelines.

#![warn(missing_docs)]

mod graph;
#[allow(clippy::module_inception)]
mod pipeline;
mod syntax;

pub mod api;
pub mod errors;
pub mod labels;
pub mod runner;

use api::{PipelineData, PipelineNode, PipelineNodeStub};

// Shortcut types

/// A [`PipelineNodeStub`]'s `NodeType`
pub type SNodeType<StubType> = <StubType as PipelineNodeStub>::NodeType;

/// A [`PipelineNodeStub`]'s `NodeType`'s `Datatype`
pub type SDataType<StubType> = <<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType;

/// The [`PipelineDataStub`] that represents a [`PipelineNodeStub`]'s `NodeType`'s `Datatype`.
pub type SDataStub<StubType> = <<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStub;

/// The [`PipelineDataStub`] that represents a [`PipelineNode`]'s `Datatype`.
pub type NDataStub<NodeType> = <<NodeType as PipelineNode>::DataType as PipelineData>::DataStub;
