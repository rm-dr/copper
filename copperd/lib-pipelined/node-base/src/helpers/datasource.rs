use copper_util::mime::MimeType;
use std::{collections::VecDeque, sync::Arc};
use url::Url;

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
	Url {
		mime: MimeType,
		url: Url,
		data: Arc<Vec<u8>>,
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

				DataSource::Url { .. } => {
					unreachable!("DataSource consumed an array after other source type")
				}
			},

			BytesSource::Url { url } => match self {
				DataSource::Uninitialized => {
					// TODO: async
					// TODO: incremental
					// TODO: handle errors
					let data = Arc::new(
						reqwest::blocking::get("https://www.rust-lang.org")
							.unwrap()
							.bytes()
							.unwrap()
							.to_vec(),
					);

					*self = DataSource::Url { mime, url, data }
				}

				_ => unreachable!("Datasource received a file after other source type"),
			},
		};
	}
}
