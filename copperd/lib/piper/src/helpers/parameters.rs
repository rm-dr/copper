use smartstring::{LazyCompact, SmartString};
use std::collections::BTreeMap;

use crate::base::{NodeParameterValue, RunNodeError};

pub struct NodeParameters {
	params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>,
}

impl From<BTreeMap<SmartString<LazyCompact>, NodeParameterValue>> for NodeParameters {
	fn from(value: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>) -> Self {
		Self::new(value)
	}
}

impl NodeParameters {
	pub fn new(params: BTreeMap<SmartString<LazyCompact>, NodeParameterValue>) -> Self {
		Self { params }
	}

	/// Return `Err(RunNodeError::UnexpectedParameter)` if we still have unhandled parameters.
	/// Otherwise, return `Ok(())`.
	pub fn err_if_not_empty(self) -> Result<(), RunNodeError> {
		if let Some((param, _)) = self.params.first_key_value() {
			return Err(RunNodeError::UnexpectedParameter {
				parameter: param.clone(),
			});
		}

		return Ok(());
	}
}

impl NodeParameters {
	pub fn pop_val(&mut self, parameter: &str) -> Result<NodeParameterValue, RunNodeError> {
		let p = self.params.remove(parameter);
		match p {
			None => {
				return Err(RunNodeError::MissingParameter {
					parameter: parameter.into(),
				});
			}

			Some(x) => return Ok(x),
		}
	}

	pub fn pop_int(&mut self, parameter: &str) -> Result<i64, RunNodeError> {
		let p = self.params.remove(parameter);
		match p {
			None => {
				return Err(RunNodeError::MissingParameter {
					parameter: parameter.into(),
				});
			}

			Some(NodeParameterValue::Integer(x)) => return Ok(x),

			Some(_) => {
				return Err(RunNodeError::BadParameterType {
					parameter: parameter.into(),
				})
			}
		}
	}

	pub fn pop_str(&mut self, parameter: &str) -> Result<SmartString<LazyCompact>, RunNodeError> {
		let p = self.params.remove(parameter);
		match p {
			None => {
				return Err(RunNodeError::MissingParameter {
					parameter: parameter.into(),
				});
			}

			Some(NodeParameterValue::String(x)) => return Ok(x),

			Some(_) => {
				return Err(RunNodeError::BadParameterType {
					parameter: parameter.into(),
				})
			}
		}
	}
}
