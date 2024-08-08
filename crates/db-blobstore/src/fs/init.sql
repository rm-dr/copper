-- Database metadata
CREATE TABLE meta (
	-- The name of the metadata variable
	var TEXT PRIMARY KEY NOT NULL UNIQUE,

	-- The value of the variable
	val TEXT NOT NULL
);

CREATE TABLE blobs (
	id INTEGER PRIMARY KEY NOT NULL,

	-- This blob's mime type
	data_type TEXT NOT NULL,

	-- A relative path to this blob's file
	file_path TEXT NOT NULL UNIQUE
);

