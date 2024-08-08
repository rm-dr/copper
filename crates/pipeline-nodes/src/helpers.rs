use async_broadcast::{broadcast, Receiver, Sender, TryRecvError, TrySendError};
use std::{
	io::{Read, Seek, SeekFrom},
	sync::Arc,
};
use ufo_pipeline::api::PipelineNodeState;

/// Write helper for `Blob` channels.
/// Handles sending & holding of messages (if the channel is full)
pub struct HoldSender {
	held_message: Option<Arc<Vec<u8>>>,
	sender: Sender<Arc<Vec<u8>>>,
}

impl HoldSender {
	pub fn new(channel_size: usize) -> (Self, Receiver<Arc<Vec<u8>>>) {
		let (sender, receiver) = broadcast(channel_size);

		(
			Self {
				held_message: None,
				sender,
			},
			receiver,
		)
	}

	pub fn is_holding(&self) -> bool {
		self.held_message.is_some()
	}

	/// If we're holding a message, try to send it.
	///
	/// If this returns `None`, keep going.
	/// If this returns `Some(_)`, return with the given status immediately.
	pub fn send_held_message(&mut self) -> Option<PipelineNodeState> {
		// If we're holding a message to send, try to send it
		if let Some(x) = self.held_message.take() {
			match self.sender.try_broadcast(x) {
				Err(TrySendError::Full(x)) => {
					// We can't send this message now, try next time.
					self.held_message = Some(x);
					return Some(PipelineNodeState::Pending);
				}
				Err(TrySendError::Inactive(_)) => {
					// This should never happen, we don't deactivate readers
					unreachable!();
					// If all readers are inactive, wait for them.
					//return Ok(PipelineNodeState::Pending);
				}
				Err(TrySendError::Closed(_)) => {
					// All readers are closed, we have no reason to keep reading this file.
					return Some(PipelineNodeState::Done);
				}
				Ok(_) => {
					// We just sent the segment we're holding, but there may be more.
					// Keep reading.
					return None;
				}
			};
		} else {
			return None;
		}
	}

	/// Send or store the given buffer.
	///
	/// If this returns `None`, keep going.
	/// If this returns `Some(_)`, return with the given status immediately.
	pub fn send_or_store(&mut self, buf: Arc<Vec<u8>>) -> Option<PipelineNodeState> {
		// This should never happen. If we have a message stored,
		// we cannot try to send another.
		assert!(self.held_message.is_none());

		// Try to send a message, store it if sending fails
		match self.sender.try_broadcast(buf) {
			Err(TrySendError::Inactive(x)) | Err(TrySendError::Full(x)) => {
				self.held_message = Some(x);
				Some(PipelineNodeState::Pending)
			}
			Err(TrySendError::Closed(_)) => Some(PipelineNodeState::Done),
			Ok(_) => None,
		}
	}
}

/// Read helper for `Blob` channels.
/// Stores a vec of `Blob` fragments and provides a `Read`
/// over a virtual, concatenated Vec<u8>
///
/// The sum of the sizes of all fragments must fit inside
/// a u64. Things will break if they don't. (TODO: fix?)
pub struct ArcVecBuffer {
	// Note the Arc. This minimizes memory use,
	// since each fragment is allocated exactly once!
	buffer: Vec<Arc<Vec<u8>>>,
	cursor: u64,
}

impl ArcVecBuffer {
	pub fn new() -> Self {
		Self {
			buffer: Vec::new(),
			cursor: 0,
		}
	}

	/// Receive all messages into this buffer.
	/// Returns (buffer_changed, all_messages_received)
	pub fn recv_all(&mut self, recv: &mut Receiver<Arc<Vec<u8>>>) -> (bool, bool) {
		let mut buffer_changed = false;
		loop {
			match recv.try_recv() {
				Err(TryRecvError::Closed) => {
					return (buffer_changed, true);
				}
				Err(TryRecvError::Empty) => {
					return (buffer_changed, false);
				}
				Err(TryRecvError::Overflowed(_)) => {
					// We never use overflowing receivers,
					// so this should never happen.
					unreachable!()
				}
				Ok(x) => {
					buffer_changed = true;
					self.buffer.push(x);
				}
			}
		}
	}

	pub fn len(&self) -> u64 {
		let mut l = 0u64;
		for b in &self.buffer {
			l += u64::try_from(b.len()).unwrap()
		}
		l
	}
}

impl Read for ArcVecBuffer {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		if self.buffer.is_empty() {
			return Ok(0);
		}

		let mut written: usize = 0;
		let mut space_left: usize = buf.len();
		loop {
			// outer idx: position in self.buffer
			// inner idx: position inside that buffer
			let (inner_idx, outer_idx) = {
				let mut i = 0;
				let mut c = self.cursor;
				loop {
					let l = u64::try_from(self.buffer[i].len()).unwrap();
					if c < l {
						break (usize::try_from(c).unwrap(), i);
					}

					i += 1;
					c -= l;

					if i >= self.buffer.len() {
						break (
							self.buffer.last().as_ref().unwrap().len(),
							self.buffer.len() - 1,
						);
					}
				}
			};

			let current = &self.buffer[outer_idx];
			let len_current = current.len();
			let current_left = len_current - inner_idx;
			let to_write = current_left.min(space_left);

			if to_write == 0 {
				return Ok(written);
			}

			buf[written..written + to_write]
				.copy_from_slice(&current[inner_idx..inner_idx + to_write]);

			self.cursor += u64::try_from(to_write).unwrap();
			written += to_write;
			space_left -= to_write;

			if space_left == 0 {
				return Ok(written);
			}
		}
	}
}

impl Seek for ArcVecBuffer {
	fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
		match pos {
			SeekFrom::Current(x) => {
				if 0 <= x {
					let x = u64::try_from(x).unwrap();
					self.cursor += x;
					if self.len() < self.cursor {
						self.cursor = self.len()
					}
				} else {
					let x = u64::try_from(x.abs()).unwrap();
					if self.cursor < x {
						return Err(std::io::ErrorKind::InvalidInput.into());
					}
					self.cursor -= x;
				}
				return Ok(self.cursor);
			}
			SeekFrom::End(x) => {
				if 0 <= x {
					self.cursor = self.len()
				} else {
					let x = u64::try_from(x.abs()).unwrap();
					if self.len() < x {
						return Err(std::io::ErrorKind::InvalidInput.into());
					}
					self.cursor = self.len() - x;
				}
				return Ok(self.cursor);
			}
			SeekFrom::Start(x) => {
				self.cursor = x;
				if self.cursor > self.len() {
					self.cursor = self.len()
				}
				return Ok(self.cursor);
			}
		}
	}
}
