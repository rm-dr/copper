//! Helper structs that contain database element properties

use argon2::{
	password_hash::{rand_core::OsRng, SaltString},
	Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
};
use copper_pipelined::json::PipelineJson;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use utoipa::ToSchema;

use crate::{PipelineId, UserId};

/// A user's hashed password.
/// This serialized for storage in the db.
///
/// It should NEVER be sent to ANYBODY.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserPassword {
	#[allow(non_camel_case_types)]
	Argon2id13_19_4 { pwhash: String },
}

impl UserPassword {
	/// Make a new hashed password from a plain string.
	/// This returns the most up-to-date variant we should use.
	pub fn new(plaintext_password: &str) -> Self {
		let salt = SaltString::generate(&mut OsRng);
		let argon2 = Argon2::new(
			Algorithm::Argon2id,
			Version::V0x13,
			Params::new(19_456, 4, 1, Some(32)).unwrap(),
		);

		let pwhash = argon2
			.hash_password(plaintext_password.as_bytes(), &salt)
			.unwrap()
			.to_string();

		return Self::Argon2id13_19_4 { pwhash };
	}

	pub fn check_password(&self, plaintext_password: &str) -> bool {
		return match self {
			Self::Argon2id13_19_4 { pwhash } => {
				let argon2 = Argon2::new(
					Algorithm::Argon2id,
					Version::V0x13,
					Params::new(19_456, 4, 1, Some(32)).unwrap(),
				);

				let hash = PasswordHash::new(pwhash).unwrap();
				argon2
					.verify_password(plaintext_password.as_bytes(), &hash)
					.is_ok()
			}
		};
	}
}

/// User Information
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserInfo {
	/// The id of this user
	#[schema(value_type = i64)]
	pub id: UserId,

	/// This user's name
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	/// This user's email
	#[schema(value_type = String)]
	pub email: SmartString<LazyCompact>,

	/// This user's hashed password.
	///
	/// DO NOT SERIALIZE THIS!
	/// This field should always be tagged with `#[serde(skip)]`
	#[serde(skip)]
	pub password: UserPassword,
}

/// Pipeline Information
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PipelineInfo {
	/// The id of this user
	#[schema(value_type = i64)]
	pub id: PipelineId,

	/// The user that owns this pipeline
	#[schema(value_type = i64)]
	pub owned_by: UserId,

	/// This user's name
	#[schema(value_type = String)]
	pub name: SmartString<LazyCompact>,

	/// The pipeline
	pub data: PipelineJson,
}
