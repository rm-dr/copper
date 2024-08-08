use crate::RouterState;
use axum::{
	extract::{Query, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::error;
use ufo_ds_core::{
	api::meta::{AttrInfo, Metastore},
	handles::ClassHandle,
};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub(super) struct ClassListRequest {
	dataset: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub(super) struct ExtendedClassInfo {
	/// This class' name
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	/// This class' unique handle
	#[schema(value_type = u32)]
	pub handle: ClassHandle,

	/// This class' attributes
	pub attrs: Vec<AttrInfo>,
}

/// Get this dataset's classes
#[utoipa::path(
	get,
	path = "/list",
	params(
		ClassListRequest
	),
	responses(
		(status = 200, description = "Classes", body = Vec<ExtendedClassInfo>),
		(status = 500, description = "Internal server error", body = String),
		(status = 401, description = "Unauthorized")
	),
)]
pub(super) async fn list_classes(
	jar: CookieJar,
	State(state): State<RouterState>,
	Query(query): Query<ClassListRequest>,
) -> Response {
	if let Err(x) = state.main_db.auth.auth_or_logout(&jar).await {
		return x;
	}

	let dataset = match state.main_db.dataset.get_dataset(&query.dataset).await {
		Ok(Some(x)) => x,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				format!("Dataset `{}` does not exist", query.dataset),
			)
				.into_response()
		}
		Err(e) => {
			error!(
				message = "Could not get dataset",
				dataset = query.dataset,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get dataset").into_response();
		}
	};

	let classes = match dataset.get_all_classes().await {
		Ok(x) => x,
		Err(e) => {
			error!(
				message = "Could not get classes",
				dataset = ?query.dataset,
				error = ?e
			);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Could not get classes").into_response();
		}
	};

	let mut out = Vec::new();
	for class in classes {
		let attrs = match dataset.class_get_attrs(class.handle).await {
			Ok(x) => x,
			Err(e) => {
				error!(
					message = "Could not get class attributes",
					dataset = ?query.dataset,
					class = ?class,
					error = ?e
				);
				return (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Could not get class attributes: {e}"),
				)
					.into_response();
			}
		};

		out.push(ExtendedClassInfo {
			name: class.name,
			handle: class.handle,
			attrs,
		});
	}

	return Json(out).into_response();
}
