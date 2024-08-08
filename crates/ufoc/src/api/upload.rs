use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use smartstring::{LazyCompact, SmartString};
use std::io::Read;
use ufo_api::upload::{UploadFinish, UploadFragmentMetadata, UploadNewFileResult, UploadStartInfo};
use ufo_util::mime::MimeType;

use super::client::UfoApiClient;
use super::errors::UfoApiError;

pub struct UfoApiUploadJob<'a> {
	pub(super) api_client: &'a UfoApiClient,
	pub(super) upload_job_id: SmartString<LazyCompact>,
}

impl<'a> UfoApiUploadJob<'a> {
	pub fn get_job_id(&self) -> &SmartString<LazyCompact> {
		&self.upload_job_id
	}

	// Upload the given file under the given job
	pub fn upload_file(
		&self,
		file_type: MimeType,
		file: &mut dyn Read,
	) -> Result<SmartString<LazyCompact>, UfoApiError> {
		let res = self
			.api_client
			.client
			.post(
				self.api_client
					.host
					.join(&format!("/upload/{}/new_file", self.upload_job_id))
					.unwrap(),
			)
			.json(&UploadStartInfo { file_type })
			.send()?;

		let new_file_info: UploadNewFileResult = serde_json::from_str(&res.text()?)?;

		let mut part_count = 0;
		let mut hasher = Sha256::new();
		loop {
			// Make sure to leave space for metadata part
			let mut buf = vec![0u8; 2 * 1024 * 1024 - (16 * 1024)];
			let n = file.read(&mut buf)?;
			buf.truncate(n);
			hasher.update(&buf);

			// Reached EOF, we're done.
			if n == 0 {
				break;
			}

			part_count += 1;
			let multipart_form = reqwest::blocking::multipart::Form::new()
				.part(
					"metadata",
					reqwest::blocking::multipart::Part::text(
						serde_json::to_string(&UploadFragmentMetadata {
							part_idx: part_count - 1,
						})
						.unwrap(),
					),
				)
				.part("fragment", reqwest::blocking::multipart::Part::bytes(buf));

			let res = self
				.api_client
				.client
				// TODO: api path enum?
				.post(
					self.api_client
						.host
						.join(&format!(
							"/upload/{}/{}",
							self.upload_job_id, new_file_info.file_name
						))
						.unwrap(),
				)
				.multipart(multipart_form)
				.send()?;

			match res.status() {
				StatusCode::OK => return Ok(new_file_info.file_name),
				StatusCode::INTERNAL_SERVER_ERROR => {
					return Err(UfoApiError::ServerError(res.text()?))
				}
				_ => unreachable!(),
			}
		}

		let hash = format!("{:X}", hasher.finalize());

		let res = self
			.api_client
			.client
			.post(
				self.api_client
					.host
					.join(&format!(
						"/upload/{}/{}/finish",
						self.upload_job_id, new_file_info.file_name
					))
					.unwrap(),
			)
			.json(&UploadFinish {
				hash: hash.into(),
				part_count,
			})
			.send()
			.unwrap();

		match res.status() {
			StatusCode::OK => return Ok(new_file_info.file_name),
			StatusCode::INTERNAL_SERVER_ERROR => return Err(UfoApiError::ServerError(res.text()?)),
			_ => unreachable!(),
		}
	}
}
