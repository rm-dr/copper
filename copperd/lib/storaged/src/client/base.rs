use std::{error::Error, fmt::Display};

use crate::{ClassId, ClassInfo, Transaction};

#[derive(Debug)]
pub enum StoragedRequestError {
	GenericHttp {
		code: u16,
		message: Option<String>,
	},
	Other {
		error: Box<dyn Error + Sync + Send + 'static>,
	},
}

impl Display for StoragedRequestError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::GenericHttp { code, message } => {
				if let Some(m) = message {
					write!(f, "Request failed with code {code}: {m}")
				} else {
					write!(f, "Request failed with code {code}")
				}
			}
			Self::Other { .. } => write!(f, "request failed"),
		}
	}
}

impl Error for StoragedRequestError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::Other { error } => Some(error.as_ref()),
			_ => None,
		}
	}
}

pub trait BlockingStoragedClient: Send + Sync {
	fn get_class(&self, class_id: ClassId) -> Result<Option<ClassInfo>, StoragedRequestError>;

	fn apply_transaction(&self, transaction: Transaction) -> Result<(), StoragedRequestError>;
}
