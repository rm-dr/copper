use crate::{helpers::maindb::dataset::DatasetType, RouterState};
use axum::{
	extract::State,
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
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
	),
)]
pub(super) async fn list_datasets(State(state): State<RouterState>) -> Response {
	let mut out = Vec::new();

	let datasets = match state.main_db.get_datasets() {
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
