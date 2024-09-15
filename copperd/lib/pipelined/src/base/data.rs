use std::fmt::Debug;

/// An immutable bit of data inside a pipeline.
///
/// These should be easy to clone. [`PipelineData`]s that
/// carry something big should wrap it in an [`std::sync::Arc`].
///
/// The [`DeserializeOwned`] implementation of this object MUST NOT be transparent.
/// See the dispatcher param enums for more details.
pub trait PipelineData
where
	Self: Debug + Clone + Send + Sync + 'static,
{
}

pub trait PipelineJobContext
where
	Self: Send + Sync + 'static,
{
}
