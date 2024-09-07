//! Errors produced by database operations

pub mod attribute;
pub mod dataset;
pub mod itemclass;

/*
#[derive(Debug)]
pub enum NewError {
	/// Database error
	DbError(Box<dyn Error + Send + Sync>),
}

impl Display for NewError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::DbError(_) => write!(f, "Database backend error"),
		}
	}
}

impl Error for NewError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			Self::DbError(x) => Some(x.as_ref()),
			_ => None,
		}
	}
}
*/
