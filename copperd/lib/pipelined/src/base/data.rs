use async_trait::async_trait;
use std::fmt::Debug;

use super::RunNodeError;

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

#[async_trait]
pub trait PipelineJobContext<DataType: PipelineData>
where
	Self: Send + Sync + 'static,
{
	async fn on_complete(self) -> Result<(), RunNodeError<DataType>>;
}
