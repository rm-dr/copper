-- Database metadata
CREATE TABLE meta (
	-- The name of the metadata variable
	var TEXT PRIMARY KEY NOT NULL UNIQUE,

	-- The value of the variable
	val TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_meta_var on meta(var);


-- Dataset index
CREATE TABLE datasets (
	id INTEGER PRIMARY KEY NOT NULL,

	-- This storage's name
	ds_name TEXT NOT NULL,

	-- This storage's type
	ds_type TEXT NOT NULL,

	-- Path to this storage
	-- (relative to dataset dir)
	ds_path TEXT NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_dataset_name on datasets(ds_name);


CREATE TABLE groups (
	id INTEGER PRIMARY KEY NOT NULL,

	group_name TEXT NOT NULL UNIQUE,
	group_permissions TEXT NOT NULL,

	-- If this is none, the parent of this group is the root group,
	-- which always has all permissions.
	group_parent INTEGER,
	FOREIGN KEY (group_parent) REFERENCES groups(id)
);

CREATE INDEX IF NOT EXISTS idx_group_name on groups(group_name);


CREATE TABLE users (
	id INTEGER PRIMARY KEY NOT NULL,

	user_name TEXT NOT NULL UNIQUE,
	pw_hash TEXT NOT NULL,

	-- The group this user belongs to.
	-- If this is NULL, this user belongs to the root group.
	user_group INTEGER,
	FOREIGN KEY (user_group) REFERENCES groups(id)
);

CREATE INDEX IF NOT EXISTS idx_user_name on users(user_name);
