use std::sync::Arc;

use axum::{routing::post, Router};
use ufo_util::mime::MimeType;
use utoipa::OpenApi;

use crate::RouterState;

mod finish;
mod new_file;
mod new_upload;
mod upload;
pub mod uploader;

use finish::*;
use new_file::*;
use new_upload::*;
use upload::*;

// TODO: better error handling
// TODO: delete when fail
// TODO: logging

#[derive(OpenApi)]
#[openapi(
	paths(start_upload, start_file, upload, finish_file),
	components(schemas(
		UploadStartResult,
		UploadStartInfo,
		UploadNewFileResult,
		UploadFragmentMetadata,
		UploadFinish,
		MimeType
	))
)]
pub(super) struct UploadApi;

pub(super) fn router(uploader: Arc<uploader::Uploader>) -> Router<RouterState> {
	let mut r = Router::new();

	let u = uploader.clone();
	r = r.route("/new", post(|| async move { start_upload(u).await }));

	let u = uploader.clone();
	r = r.route(
		"/:job_id/newfile",
		post(|path, payload| async move { start_file(u, path, payload).await }),
	);

	let u = uploader.clone();
	r = r.route(
		"/:job_id/:file_handle",
		post(|path, multipart| async move { upload(u, path, multipart).await }),
	);

	let u = uploader.clone();
	r = r.route(
		"/:job_id/:file_id/finish",
		post(|path, payload| async move { finish_file(u, path, payload).await }),
	);

	return r;
}
