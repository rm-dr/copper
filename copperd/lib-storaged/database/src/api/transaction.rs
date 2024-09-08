//! Definitions for high-level dataset transactions
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
	data::AttrData,
	handles::{AttributeId, ClassId},
};

/// A single action in a transaction
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum TransactionAction {
	/// Add an item
	AddItem {
		/// The class to add the item to
		#[schema(value_type = u32)]
		to_class: ClassId,

		/// The attributes to create the item with
		attributes: Vec<(AttributeId, AttrData)>,
	},
}

/// A set of actions to apply to a dataset.
///
/// Transactions are atomic: they either fully succeed or fully fail.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Transaction {
	/// The actions to apply.
	/// These are applied in an arbitrary order, possibly in parallel.
	pub actions: Vec<TransactionAction>,
}
