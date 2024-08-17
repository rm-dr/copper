use axum::{routing::post, Router};
use utoipa::OpenApi;

use super::RouterState;

mod finish;
mod new_file;
mod new_upload;
#[allow(clippy::module_inception)]
mod upload;

use finish::*;
use new_file::*;
use new_upload::*;
use upload::*;

#[derive(OpenApi)]
#[openapi(
	paths(start_upload, start_file, upload, finish_file),
	components(schemas(
		UploadStartResult,
		UploadStartInfo,
		UploadNewFileResult,
		UploadFragmentMetadata,
		UploadFinish,
	))
)]
pub(super) struct UploadApi;

pub(super) fn router() -> Router<RouterState> {
	return Router::new()
		.route("/new", post(start_upload))
		.route("/:job_id/newfile", post(start_file))
		.route("/:job_id/:file_handle", post(upload))
		.route("/:job_id/:file_id/finish", post(finish_file));
}
