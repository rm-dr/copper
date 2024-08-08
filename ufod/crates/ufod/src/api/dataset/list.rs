use crate::RouterState;
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing::error;
use ufo_ds_impl::DatasetType;
use utoipa::ToSchema;

/// Dataset info
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub(super) struct DatasetInfoShort {
	/// This dataset's name
	pub name: String,

	/// This dataset's type
	pub ds_type: DatasetType,
}

/// Get all datasets
#[utoipa::path(
	get,
	path = "/list",
	responses(
		(status = 200, description = "Dataset info", body = Vec<DatasetInfoShort>),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
	// For header auth
	//security(
	//	("bearer" = []),
	//)
)]
pub(super) async fn list_datasets(
	jar: CookieJar,
	//headers: HeaderMap,
	State(state): State<RouterState>,
) -> Response {
	match state.main_db.auth.check_headers(&jar).await {
		Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
		Ok(Some(_)) => {}
		Err(e) => {
			error!(
				message = "Could not check auth cookies",
				cookies = ?jar,
				error = ?e
			);
			return (
				StatusCode::INTERNAL_SERVER_ERROR,
				format!("Could not check auth cookies"),
			)
				.into_response();
		}
	}

	let mut out = Vec::new();

	let datasets = match state.main_db.dataset.get_datasets().await {
		Ok(x) => x,
		Err(e) => {
			error!(
				message = "Could not get all datasets",
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")).into_response();
		}
	};

	for ds in datasets {
		out.push(DatasetInfoShort {
			name: ds.name.to_string(),
			ds_type: ds.ds_type,
		})
	}

	return (StatusCode::OK, Json(out)).into_response();
}
