use axum::{
	extract::{Path, State},
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use axum_extra::extract::CookieJar;
use copper_itemdb::{client::base::client::ItemdbClient, AttrData, ClassId, ItemId};
use copper_jobqueue::base::errors::AddJobError;
use copper_util::HashType;
use serde::Deserialize;
use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;
use tracing::error;
use utoipa::ToSchema;

use crate::{
	database::base::{client::DatabaseClient, errors::pipeline::GetPipelineError},
	uploader::{errors::UploadAssignError, GotJobKey, UploadJobId},
	RouterState,
};

/// Attribute data, provided by the user by api calls.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub(super) enum ApiInputAttrData {
	/// A block of text
	Text {
		#[schema(value_type = String)]
		value: SmartString<LazyCompact>,
	},

	/// An integer
	Integer {
		/// The integer
		value: i64,

		/// If true, this integer must be non-negative
		is_non_negative: bool,
	},

	/// A float
	Float {
		/// The float
		value: f64,

		/// If true, this float must be non-negative
		is_non_negative: bool,
	},

	/// A boolean
	Boolean { value: bool },

	/// A checksum
	Hash {
		/// The type of this hash
		hash_type: HashType,

		/// The hash data
		data: Vec<u8>,
	},

	/// Binary data we uploaded previously
	Blob {
		/// The upload id. This must only be used once,
		/// uploaded files are deleted once their job is done.
		///
		/// Also, note that we _never_ send the S3 key to the
		/// client---only the upload id as a proxy. This makes sure
		/// that clients can only start jobs on uploads they own,
		/// and reduces the risk of other creative abuse.
		#[schema(value_type = String)]
		upload_id: UploadJobId,
	},

	/// A reference to an item in another class
	Reference {
		/// The item class this reference points to
		#[schema(value_type = i64)]
		class: ClassId,

		/// The item
		#[schema(value_type = i64)]
		item: ItemId,
	},
}

impl TryFrom<&ApiInputAttrData> for AttrData {
	type Error = ();

	fn try_from(value: &ApiInputAttrData) -> Result<Self, Self::Error> {
		value.clone().try_into()
	}
}

impl TryFrom<ApiInputAttrData> for AttrData {
	type Error = ();

	fn try_from(value: ApiInputAttrData) -> Result<Self, Self::Error> {
		Ok(match value {
			ApiInputAttrData::Blob { .. } => return Err(()),

			ApiInputAttrData::Boolean { value } => Self::Boolean { value },
			ApiInputAttrData::Text { value } => Self::Text { value },
			ApiInputAttrData::Hash { hash_type, data } => Self::Hash { hash_type, data },
			ApiInputAttrData::Reference { class, item } => Self::Reference { class, item },

			ApiInputAttrData::Float {
				value,
				is_non_negative,
			} => Self::Float {
				value,
				is_non_negative,
			},

			ApiInputAttrData::Integer {
				value,
				is_non_negative,
			} => Self::Integer {
				value,
				is_non_negative,
			},
		})
	}
}

#[derive(Deserialize, ToSchema, Debug)]
pub(super) struct RunPipelineRequest {
	/// A unique id for this job
	#[schema(value_type = String)]
	pub job_id: SmartString<LazyCompact>,

	#[schema(value_type = BTreeMap<String, ApiInputAttrData>)]
	pub input: BTreeMap<SmartString<LazyCompact>, ApiInputAttrData>,
}

/// Start a pipeline job
#[utoipa::path(
	post,
	path = "/{pipeline_id}/run",
	params(
		("pipeline_id", description = "Pipeline id"),
	),
	responses(
		(status = 200, description = "Job queued successfully"),
		(status = 401, description = "Unauthorized"),
		(status = 409, description = "Job id already exists"),
		(status = 429, description = "Job queue is full"),
	),
	security(
		("bearer" = []),
	)
)]
pub(super) async fn run_pipeline<Client: DatabaseClient, Itemdb: ItemdbClient>(
	jar: CookieJar,
	State(state): State<RouterState<Client, Itemdb>>,
	Path(pipeline_id): Path<i64>,
	Json(payload): Json<RunPipelineRequest>,
) -> Response {
	let user = match state.auth.auth_or_logout(&state, &jar).await {
		Err(x) => return x,
		Ok(user) => user,
	};

	let pipe = match state.db_client.get_pipeline(pipeline_id.into()).await {
		Ok(Some(pipe)) => pipe,
		Ok(None) => return StatusCode::NOT_FOUND.into_response(),
		Err(GetPipelineError::DbError(error)) => {
			error!(
				message = "Database error while getting pipeline",
				?pipeline_id,
				?error,
			);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}
	};

	// Users can only get pipelines they own
	if pipe.owned_by != user.id {
		return StatusCode::UNAUTHORIZED.into_response();
	}

	let mut converted_input: BTreeMap<SmartString<LazyCompact>, AttrData> = BTreeMap::new();
	for (k, v) in payload.input {
		// If we can automatically convert, do so
		if let Ok(x) = AttrData::try_from(&v) {
			converted_input.insert(k, x);
			continue;
		}

		// Some types need manual conversion
		if let Some(x) = match &v {
			ApiInputAttrData::Blob { upload_id } => {
				let res = state.uploader.get_job_object_key(user.id, upload_id).await;
				let key = match res {
					GotJobKey::NoSuchJob => {
						return (
							StatusCode::BAD_REQUEST,
							Json(format!(
								"Invalid input: input {k} references a job that does not exist"
							)),
						)
							.into_response();
					}

					GotJobKey::JobNotDone => {
						return (
							StatusCode::BAD_REQUEST,
							Json(format!(
								"Invalid input: input {k} references a job that is not finished"
							)),
						)
							.into_response();
					}

					GotJobKey::JobIsAssigned => {
						return (
							StatusCode::BAD_REQUEST,
							Json(format!(
								"Invalid input: input {k} references a job that has been assigned to a pipeline"
							)),
						)
							.into_response();
					}

					GotJobKey::HereYouGo(key) => key,
				};

				let res = state
					.uploader
					.assign_job_to_pipeline(user.id, upload_id, &payload.job_id)
					.await;

				match res {
					// This is impossible, we already checked these cases
					Err(UploadAssignError::BadUpload) => unreachable!(),
					Err(UploadAssignError::NotMyUpload) => unreachable!(),

					Ok(()) => Some(AttrData::Blob {
						bucket: (&state.config.edged_objectstore_upload_bucket).into(),
						key,
					}),
				}
			}

			_ => None,
		} {
			converted_input.insert(k, x);
			continue;
		}

		unreachable!("User-provided data {v:?} could not be converted automatically, but was not caught by the manual conversion `match`.")
	}

	let res = state
		.jobqueue_client
		.add_job(
			payload.job_id.as_str().into(),
			user.id,
			&pipe.data,
			&converted_input,
		)
		.await;

	return match res {
		Ok(_) => StatusCode::OK.into_response(),

		Err(AddJobError::DbError(error)) => {
			error!(message = "DB error while queueing job", ?error, ?payload.job_id);
			return StatusCode::INTERNAL_SERVER_ERROR.into_response();
		}

		Err(AddJobError::AlreadyExists) => {
			return StatusCode::CONFLICT.into_response();
		}
	};
}
