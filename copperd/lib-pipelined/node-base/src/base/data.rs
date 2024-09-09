use serde::de::DeserializeOwned;
use smartstring::{LazyCompact, SmartString};
use std::{collections::BTreeMap, fmt::Debug};

/// An immutable bit of data inside a pipeline.
///
/// These should be easy to clone. [`PipelineData`]s that
/// carry something big should wrap it in an [`std::sync::Arc`].
///
/// The `Deserialize` implementation of this struct MUST NOT be transparent.
/// It should always be some sort of object. See the dispatcher param enums
/// for more details.
pub trait PipelineData
where
	Self: DeserializeOwned + Debug + Clone + Send + Sync + 'static,
{
	/// The stub type that represents this node.
	type DataStubType: PipelineDataStub;

	/// Transform this data container into its type.
	fn as_stub(&self) -> Self::DataStubType;
}

/// A "type" of [`PipelineData`].
///
/// This does NOT carry data. Rather, it tells us
/// what *kind* of data a pipeline inputs/outputs.
///
/// The `Deserialize` implementation of this struct MUST NOT be transparent.
/// It should always be some sort of object. See the dispatcher param enums
/// for more details.
pub trait PipelineDataStub
where
	Self: DeserializeOwned + Debug + PartialEq + Eq + Clone + Copy + Send + Sync + 'static,
{
	/// If true, an input of type `superset` can accept data of type `self`.
	fn is_subset_of(&self, superset: &Self) -> bool;
}

/// Arbitrary additional information for a pipeline job.
pub trait PipelineJobContext<DataType>
where
	Self: Send + Sync + 'static,
	DataType: PipelineData,
{
	/// Get the inputs we passed to this pipeline
	fn get_input(&self) -> &BTreeMap<SmartString<LazyCompact>, DataType>;
}
