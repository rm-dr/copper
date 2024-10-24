//! Definitions for high-level dataset transactions
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};

use crate::{AttrData, AttrDataStub, AttributeId, ClassId};

//
// MARK: Errors
//

/// An error we can encounter when creating an item
#[derive(Debug, Serialize, Deserialize)]
pub enum AddItemError {
	/// We tried to add an item to a class that doesn't exist
	NoSuchClass,

	/// We tried to create an item that contains an
	/// attribute that doesn't exist
	BadAttribute,

	/// We tried to create an item,
	/// but provided multiple values for one attribute
	RepeatedAttribute,

	/// We tried to assign data to an attribute,
	/// but that data has the wrong type
	AttributeDataTypeMismatch,

	/// We tried to create an item that contains an
	/// attribute from another class
	ForeignAttribute,

	/// We tried to create an item with attribute that violate a "not null" constraint
	NotNullViolated,

	/// We tried to create an item with attribute that violate a "unique" constraint
	UniqueViolated,
}

impl Display for AddItemError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NoSuchClass => write!(f, "tried to add an item to a class that doesn't exist"),
			Self::ForeignAttribute => write!(f, "tried to create an item with a foreign attribute"),

			Self::BadAttribute => {
				write!(f, "tried to create an item an attribute that doesn't exist")
			}

			Self::RepeatedAttribute => {
				write!(f, "multiple values were provided for one attribute")
			}

			Self::AttributeDataTypeMismatch => {
				write!(
					f,
					"tried to assign data to an attribute, but type doesn't match"
				)
			}

			Self::NotNullViolated => {
				write!(
					f,
					"tried to create an item with attributes that violate a `not null` constraint"
				)
			}

			Self::UniqueViolated => {
				write!(
					f,
					"tried to create an item with attributes that violate a `unique` constraint"
				)
			}
		}
	}
}

impl Error for AddItemError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		None
	}
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
