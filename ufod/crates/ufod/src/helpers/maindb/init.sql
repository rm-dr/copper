-- Database metadata
CREATE TABLE meta (
	-- The name of the metadata variable
	var TEXT PRIMARY KEY NOT NULL UNIQUE,

	-- The value of the variable
	val TEXT NOT NULL
);


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
