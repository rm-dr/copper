use std::{collections::VecDeque, sync::Arc};
use ufo_util::mime::MimeType;

use crate::data::BytesSource;

/// An opened data source.
/// This is used inside nodes to help read `Bytes` data types.
pub enum DataSource {
	Uninitialized,
	Binary {
		mime: MimeType,
		data: VecDeque<Arc<Vec<u8>>>,
		is_done: bool,
	},
	File {
		mime: MimeType,
		file: std::fs::File,
	},
}

impl DataSource {
	pub fn consume(&mut self, mime: MimeType, source: BytesSource) {
		match source {
			BytesSource::Array { is_last, fragment } => match self {
				DataSource::Uninitialized => {
					*self = DataSource::Binary {
						mime,
						data: VecDeque::from([fragment]),
						is_done: is_last,
					}
				}

				DataSource::Binary {
					mime: current_mime,
					data,
					is_done,
				} => {
					assert!(!*is_done);
					assert!(mime == *current_mime);
					data.push_back(fragment);
					*is_done = is_last;
				}

				// TODO: proper errors
				_ => panic!(""),
			},

			BytesSource::File { path } => match self {
				DataSource::Uninitialized => {
					*self = DataSource::File {
						mime,
						file: std::fs::File::open(path).unwrap(),
					}
				}

				_ => panic!(""),
			},
		};
	}
}
