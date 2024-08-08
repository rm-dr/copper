use ufo_pipeline::api::{PipelineData, PipelineJobContext};

pub mod blob;
pub mod meta;
pub mod pipe;

pub trait Dataset<DataType: PipelineData, ContextType: PipelineJobContext<DataType>>
where
	Self: blob::Blobstore + meta::Metastore + pipe::Pipestore<DataType, ContextType> + Send + Sync,
{
}
