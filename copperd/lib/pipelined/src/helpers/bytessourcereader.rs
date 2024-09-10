use copper_util::MimeType;
use std::{collections::VecDeque, sync::Arc};

use crate::data::BytesSource;

pub struct BytesSourceArrayReader {
	pub mime: MimeType,
	pub data: VecDeque<Arc<Vec<u8>>>,
	pub is_done: bool,
}

impl BytesSourceArrayReader {
	pub fn new(mime: Option<MimeType>, source: BytesSource) -> Option<Self> {
		match source {
			BytesSource::Array { is_last, fragment } => {
				return Some(Self {
					mime: mime.unwrap_or(MimeType::Blob),
					data: VecDeque::from([fragment]),
					is_done: is_last,
				})
			}

			_ => return None,
		};
	}

	pub fn consume(&mut self, source: BytesSource) {
		match source {
			BytesSource::Array { is_last, fragment } => {
				assert!(self.is_done);
				self.data.push_back(fragment);
				self.is_done = is_last;
			}

			_ => unreachable!("consumed a non-Array source"),
		};
	}
}
