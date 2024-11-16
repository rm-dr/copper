use async_trait::async_trait;
use copper_util::MimeType;
use std::{fmt::Debug, sync::Arc};
use tokio::{
	sync::mpsc::{Receiver, Sender},
	task::JoinSet,
};

use super::rawbytes::{RawBytesSource, RawBytesSourceReader};
use crate::{
	base::{NodeId, RunNodeError},
	CopperContext,
};

/// An uninitialized [`BytesProcessor`]
#[derive(Debug, Clone)]
pub struct BytesProcessorBuilder {
	/// The source of bytes to process
	source: RawBytesSource,

	/// A list of processors to run on this stream.
	builders: Vec<Arc<dyn StreamProcessorBuilder>>,
}

unsafe impl Send for BytesProcessorBuilder {}
unsafe impl Sync for BytesProcessorBuilder {}

impl BytesProcessorBuilder {
	pub fn new(source: RawBytesSource) -> Self {
		Self {
			source,
			builders: Vec::new(),
		}
	}

	pub fn add_processor(mut self, spb: Arc<dyn StreamProcessorBuilder>) -> Self {
		self.builders.push(spb);
		return self;
	}

	pub async fn build(&self, ctx: &CopperContext<'_>) -> Result<BytesProcessor, RunNodeError> {
		let mut tasks = JoinSet::new();
		let max_buffer_size = ctx.stream_fragment_size;

		// Initial pipe
		let (s, mut receiver) = tokio::sync::mpsc::channel(ctx.stream_channel_size);

		// Start raw source reader task
		let mut reader = RawBytesSourceReader::open(ctx, self.source.clone()).await?;
		let mut mime = reader.mime().clone();
		tasks.spawn(async move {
			loop {
				let res = reader.next_fragment(max_buffer_size).await;
				match res {
					Err(e) => return Err(e),
					Ok(Some(x)) => match s.send(x).await {
						Ok(()) => {}

						// Receiver was dropped, exit early
						Err(_) => return Ok(()),
					},
					Ok(None) => return Ok(()),
				}
			}
		});

		// Start each processor task
		for b in &self.builders {
			let sp = b.build();
			mime = sp.mime().clone();

			let (ns, mut nr) = tokio::sync::mpsc::channel(ctx.stream_channel_size);
			std::mem::swap(&mut receiver, &mut nr);

			tasks.spawn(async move { sp.run(nr, ns, max_buffer_size).await });
		}

		return Ok(BytesProcessor {
			tasks,
			receiver,
			mime,
		});
	}
}

/// An initialized stream of processed bytes.
/// This is what we use to read all byte streams.
pub struct BytesProcessor {
	mime: MimeType,
	tasks: JoinSet<Result<(), RunNodeError>>,
	receiver: Receiver<Arc<Vec<u8>>>,
}

impl BytesProcessor {
	/// Get the type of data this processor produces
	pub fn mime(&self) -> &MimeType {
		&self.mime
	}

	/// Get processed data from this stream processor.
	///
	/// If this method returns none, this processor is closed and will never produce any more data.
	/// If this method returns an empty vec, data is not available yet but may be available later.
	pub async fn next_fragment(&mut self) -> Result<Option<Arc<Vec<u8>>>, RunNodeError> {
		match self.tasks.try_join_next() {
			None => {}
			Some(Ok(Ok(()))) => {}
			Some(Ok(Err(err))) => return Err(err),
			Some(Err(err)) => return Err(err.into()),
		}

		match self.receiver.recv().await {
			None => return Ok(None),
			Some(x) => return Ok(Some(x)),
		}
	}
}

/// A helper struct that creates [`StreamProcessor`]s
/// We need this because [`BytesSource`] will be cloned
/// (once for each output edge) before any processors are run.
pub trait StreamProcessorBuilder: Debug {
	fn build(&self) -> Box<dyn StreamProcessor>;
}

/// An object that consumes one stream and produces another
#[async_trait]
pub trait StreamProcessor: Send + Sync {
	/// Get the type of data this processor produces
	fn mime(&self) -> &MimeType;

	/// Get this stream processor's name
	fn name(&self) -> &'static str;

	/// Return the id of the node that created this processor
	fn source_node_id(&self) -> &NodeId;

	/// Return the type of the node that created this processor
	fn source_node_type(&self) -> &str;

	/// Run this stream processor. This is run in a separate task,
	/// and should resolve when `source` runs out of data, or when
	/// all possible output has been sent to `sink`.
	async fn run(
		&self,
		source: Receiver<Arc<Vec<u8>>>,
		sink: Sender<Arc<Vec<u8>>>,
		max_buffer_size: usize,
	) -> Result<(), RunNodeError>;
}
