use sea_orm::DbErr;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum SeaDatasetError {
	/// We haven't connected to this database yet
	NotConnected,

	/// SQL error
	Database(DbErr),

	/// We were given a bad attribute handle
	BadAttrHandle,

	/// We tried to set an attribute with data of a different type
	TypeMismatch,

	/// A `unique` constraint was violated
	UniqueViolated,
}

impl Display for SeaDatasetError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotConnected => write!(f, "NotConnected"),
			Self::Database(dberr) => write!(f, "DB Error: {}", dberr),
			Self::BadAttrHandle => write!(f, "BadAttrHandle"),
			Self::TypeMismatch => write!(f, "TypeMismatch"),
			Self::UniqueViolated => write!(f, "UniqueViolated"),
		}
	}
}

impl Error for SeaDatasetError {}
