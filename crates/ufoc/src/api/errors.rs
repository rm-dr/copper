use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum UfoApiError {
	BadJson(serde_json::Error),
	IoError(std::io::Error),
	NetworkError(reqwest::Error),
	ServerError(String),
	BadRequest(String),
}

impl Display for UfoApiError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NetworkError(_) => write!(f, "network error"),
			Self::BadJson(_) => write!(f, "bad json from server"),
			Self::IoError(_) => write!(f, "i/o error while manipulating local file"),
			Self::ServerError(s) => write!(f, "internal server error: `{s}`"),
			Self::BadRequest(s) => write!(f, "bad request: `{s}`"),
		}
	}
}

impl Error for UfoApiError {
	fn cause(&self) -> Option<&dyn Error> {
		match self {
			Self::NetworkError(e) => Some(e),
			Self::BadJson(e) => Some(e),
			Self::IoError(e) => Some(e),
			_ => None,
		}
	}
}

impl From<reqwest::Error> for UfoApiError {
	fn from(value: reqwest::Error) -> Self {
		Self::NetworkError(value)
	}
}

impl From<std::io::Error> for UfoApiError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<serde_json::Error> for UfoApiError {
	fn from(value: serde_json::Error) -> Self {
		Self::BadJson(value)
	}
}
