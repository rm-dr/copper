use ufo_pipeline::api::PipelineNodeStub;

pub mod blob;
pub mod meta;
pub mod pipe;

pub trait Dataset<PipelineNodeStubType: PipelineNodeStub>
where
	Self: blob::Blobstore + meta::Metastore + pipe::Pipestore<PipelineNodeStubType> + Send + Sync,
{
}
