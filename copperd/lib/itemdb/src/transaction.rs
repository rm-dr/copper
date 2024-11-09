//! Definitions for high-level dataset transactions
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{AttrData, AttrDataStub, AttributeId, ClassId, ItemId};

//
// MARK: Errors
//

/// An error we can encounter when creating an item
#[derive(Debug, Serialize, Deserialize, Error)]
pub enum AddItemError {
	/// We tried to add an item to a class that doesn't exist
	#[error("tried to add an item to a class that doesn't exist")]
	NoSuchClass,

	/// We tried to create an item that contains an
	/// attribute that doesn't exist
	#[error("tried to create an item an attribute that doesn't exist")]
	BadAttribute,

	/// We tried to create an item,
	/// but provided multiple values for one attribute
	#[error("multiple values were provided for one attribute")]
	RepeatedAttribute,

	/// We tried to assign data to an attribute,
	/// but that data has the wrong type
	#[error("tried to assign data to an attribute, but type doesn't match")]
	AttributeDataTypeMismatch,

	/// We tried to create an item that contains an
	/// attribute from another class
	#[error("tried to create an item with a foreign attribute")]
	ForeignAttribute,

	/// We tried to create an item with attribute that violate a "not null" constraint
	#[error("tried to create an item with attributes that violate a `not null` constraint")]
	NotNullViolated,

	/// We tried to create an item with attribute that violate a "unique" constraint
	#[error("tried to create an item with attributes that violate a `unique` constraint")]
	UniqueViolated { conflicting_ids: Vec<ItemId> },
}

//
// MARK: transactions
//

/// A value computed from a previous transaction,
/// or a value provided directly.
#[derive(Debug)]
pub enum ResultOrDirect<T> {
	Result {
		action_idx: usize,
		expected_type: AttrDataStub,
	},
	Direct {
		value: T,
	},
}

impl<T> From<T> for ResultOrDirect<T> {
	fn from(value: T) -> Self {
		Self::Direct { value }
	}
}

#[derive(Debug)]
pub enum OnUniqueConflictAction {
	/// Fail the transaction
	Fail,

	/// Do not add an item, return the item we conflicted with.
	/// If we conflicted with one item, return its id.
	/// If we conflicted with more than one, fail.
	ConflictIdOrFail,
}

/// A single action in a transaction
#[derive(Debug)]
pub enum TransactionAction {
	/// Add an item
	AddItem {
		/// The class to add the item to.
		/// The transaction will fail if this is not a valid class id.
		to_class: ClassId,

		/// The attributes to create the item with.
		///
		/// Each attribute may be directly provided, or
		/// computed from the result of a previous transaction.
		attributes: Vec<(AttributeId, ResultOrDirect<AttrData>)>,

		/// What to do if we encounter a "unique" attribute conflict
		on_unique_conflict: OnUniqueConflictAction,
	},
}

impl TransactionAction {
	pub fn result_type(&self) -> Option<AttrDataStub> {
		match self {
			Self::AddItem { to_class, .. } => Some(AttrDataStub::Reference { class: *to_class }),
		}
	}
}

/// A set of actions to apply to a dataset.
///
/// Transactions are atomic: they either fully succeed or fully fail.
#[derive(Debug)]
pub struct Transaction {
	/// The actions to apply.
	/// These must be applied in order.
	actions: Vec<TransactionAction>,
}

impl Transaction {
	pub fn new() -> Self {
		Self {
			actions: Vec::new(),
		}
	}

	pub fn add_action(&mut self, action: TransactionAction) -> usize {
		let idx = self.actions.len();
		self.actions.push(action);
		return idx;
	}

	pub fn is_empty(&self) -> bool {
		return self.actions.is_empty();
	}

	pub fn len(&self) -> usize {
		return self.actions.len();
	}
}

impl IntoIterator for Transaction {
	type Item = TransactionAction;
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		return self.actions.into_iter();
	}
}
