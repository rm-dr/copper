-- Database metadata
CREATE TABLE meta (
	-- The name of the metadata variable
	var TEXT PRIMARY KEY NOT NULL UNIQUE,

	-- The value of the variable
	val TEXT NOT NULL
);

CREATE INDEX idx_meta_var on meta(var);



-- Users
CREATE TABLE user (
	id INTEGER PRIMARY KEY NOT NULL,

	-- The email this user logs in with
	user_email TEXT NOT NULL UNIQUE,

	-- This user's display name
	user_name TEXT NOT NULL,

	-- This user's hashed & salted password
	user_pass TEXT NOT NULL
);

CREATE INDEX user_email on user(user_email);
