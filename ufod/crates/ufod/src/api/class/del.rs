use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use itertools::Itertools;
use serde::Deserialize;
use tracing::error;
use ufo_ds_core::{api::meta::Metastore, errors::MetastoreError, handles::ClassHandle};
use utoipa::ToSchema;

use crate::api::RouterState;

#[derive(Deserialize, ToSchema, Debug)]
pub(in crate::api) struct DelClassRequest {
	pub dataset: String,

	#[schema(value_type = u32)]
	pub class: ClassHandle,
}

/// Delete a class and all data associated with it
#[utoipa::path(
	delete,
	path = "/del",
	responses(
		(status = 200, description = "Successfully deleted this class"),
		(status = 400, description = "Invalid request", body=String),
		(status = 404, description = "Invalid dataset or class", body=String),
		(status = 500, description = "Internal server error", body=String),
	),
)]
pub(super) async fn del_class(
	jar: CookieJar,
	State(state): State<RouterState>,
	Json(payload): Json<DelClassRequest>,
) -> Response {
	match state.main_db.auth.auth_or_logout(&jar).await {
		Err(x) => return x,
		Ok(_) => {}
	}

	let dataset = match state.main_db.dataset.get_dataset(&payload.dataset).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{}` does not exist", payload.dataset),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset",
				dataset = payload.dataset,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get dataset"),
			)
				.into_response();
		}
	};

	let class = match dataset.get_class(payload.class).await {
		Ok(x) => x,
		Err(MetastoreError::BadClassHandle) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Class `{:?}` does not exist", payload.class),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get class",
				dataset = payload.dataset,
				class = ?payload.class,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not get class"),
			)
				.into_response();
		}
	};

	let res = dataset.del_class(class.handle).await;

	match res {
		Ok(_) => return StatusCode::OK.into_response(),
		Err(MetastoreError::DeleteClassDanglingRef(data)) => {
			let backlink_string = match data.len() {
				0 => unreachable!(),
				1 => format!("`{}`", data[0]),
				2 => format!("`{}` and `{}`", data[0], data[1]),
				x => format!(
					"{}, and `{}`",
					data[x..data.len() - 1]
						.iter()
						.map(|x| format!("`{x}`"))
						.join(", "),
					data.last().unwrap()
				),
			};

			return (
				StatusCode::BAD_REQUEST,
				format!("We cannot delete this class because there are references to it in {backlink_string}."),
			)
				.into_response();
		}
		Err(e) => {
			error!(
				message = "Could not delete class",
				dataset = payload.dataset,
				class_name = ?payload.class,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not delete class: {e}"),
			)
				.into_response();
		}
	}
}
