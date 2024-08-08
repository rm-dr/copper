//! Fast, flexible, parallel data processing pipelines.

#![warn(missing_docs)]

mod graph;

pub mod api;
pub mod labels;
#[allow(clippy::module_inception)]
pub mod pipeline;
pub mod runner;

use api::{PipelineData, PipelineNode, PipelineNodeStub};

// Shortcut types

/// A [`PipelineNodeStub`]'s `NodeType`
pub type SNodeType<StubType> = <StubType as PipelineNodeStub>::NodeType;

/// A [`PipelineNodeStub`]'s `NodeType`'s `Datatype`
pub type SDataType<StubType> = <<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType;

/// The error a [`PipelineNodeStub`] produces when given bad input
pub type SStubErrorType<StubType> = <StubType as PipelineNodeStub>::ErrorType;

/// The [`PipelineDataStub`] that represents a [`PipelineNodeStub`]'s `NodeType`'s `Datatype`.
pub type SDataStub<StubType> = <<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::DataType as PipelineData>::DataStubType;

/// The error that a [`PipelineNodeStub`]'s `NodeType` produces.
pub type SNodeErrorType<StubType> =
	<<StubType as PipelineNodeStub>::NodeType as PipelineNode>::ErrorType;

/// The [`PipelineDataStub`] that represents a [`PipelineNode`]'s `Datatype`.
pub type NDataStub<NodeType> = <<NodeType as PipelineNode>::DataType as PipelineData>::DataStubType;
