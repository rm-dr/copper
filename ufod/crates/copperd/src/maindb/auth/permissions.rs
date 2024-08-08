use serde::{Deserialize, Serialize};

use super::GroupId;

//
//
// Serialized permissions
//
//

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) enum SerializedGroupPermissionState {
	Transparent,
	Disallowed,
}

impl Default for SerializedGroupPermissionState {
	fn default() -> Self {
		Self::Disallowed
	}
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
	t == &T::default()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct SerializedGroupPermissions {
	#[serde(default, skip_serializing_if = "is_default")]
	edit_datasets: SerializedGroupPermissionState,

	#[serde(default, skip_serializing_if = "is_default")]
	edit_users_sub: SerializedGroupPermissionState,

	#[serde(default, skip_serializing_if = "is_default")]
	edit_users_same: SerializedGroupPermissionState,

	#[serde(default, skip_serializing_if = "is_default")]
	edit_groups: SerializedGroupPermissionState,
}

//
//
// Active permissions
//
//

#[derive(Debug, Clone)]
pub enum GroupPermissionState {
	Allowed,
	Disallowed { by: GroupId },
}

impl GroupPermissionState {
	pub(super) fn overlay(&mut self, other: &SerializedGroupPermissionState, other_group: GroupId) {
		match (&self, other) {
			(Self::Allowed, SerializedGroupPermissionState::Transparent) => *self = Self::Allowed,
			(Self::Allowed, SerializedGroupPermissionState::Disallowed) => {
				*self = Self::Disallowed { by: other_group }
			}
			(Self::Disallowed { .. }, _) => return,
		}
	}

	pub fn is_allowed(&self) -> bool {
		matches!(self, Self::Allowed)
	}
}

#[derive(Debug, Clone)]
pub struct GroupPermissions {
	pub edit_datasets: GroupPermissionState,

	/// Are we allowed to edit users in sibgroups of this group?
	pub edit_users_sub: GroupPermissionState,

	/// Are we allowed to edit users in THIS group?
	/// This always disallowed if `edit_users_sub` is disallowed.
	pub edit_users_same: GroupPermissionState,

	pub edit_groups: GroupPermissionState,
}

impl GroupPermissions {
	/// Make a new set of permissions for the root group. (i.e, where all actions are allowed)
	/// All permissions are created by overlaying parents on the root group.
	pub(super) fn new_root() -> Self {
		Self {
			edit_datasets: GroupPermissionState::Allowed,
			edit_users_sub: GroupPermissionState::Allowed,
			edit_users_same: GroupPermissionState::Allowed,
			edit_groups: GroupPermissionState::Allowed,
		}
	}

	/// Modify this group by passing it through a filter.
	///
	/// Any permissions disallowed in `filter_group` will be disallowed in this group.
	/// Any permissions already disallowed in this group will not be changed.
	///
	/// `overlay` always produces a group that is "weaker-or-equal-to" `self`.
	pub(super) fn overlay(
		mut self,
		filter: &SerializedGroupPermissions,
		filter_group: GroupId,
	) -> Self {
		self.edit_datasets
			.overlay(&filter.edit_datasets, filter_group);
		self.edit_groups
			.overlay(&filter.edit_datasets, filter_group);
		self.edit_users_sub
			.overlay(&filter.edit_users_sub, filter_group);
		self.edit_users_same
			.overlay(&filter.edit_users_same, filter_group);
		return self;
	}
}
