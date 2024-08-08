use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
pub mod local;

/// All types of dataset implementations we provide in this crate
///
/// We use this to keep track of dataset types in our db,
/// and in some API endpoints.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, ToSchema)]
pub enum DatasetType {
	Local,
}
