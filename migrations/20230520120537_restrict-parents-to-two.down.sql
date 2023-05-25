DROP TABLE parent_name_preferences;

DELETE FROM parent_name_preference_values WHERE value = 'neutral';

CREATE TABLE parents (
	parent TEXT PRIMARY KEY
);

CREATE TABLE parent_name_preferences (
	name TEXT NOT NULL
		REFERENCES names(name)
		ON DELETE CASCADE,
	parent TEXT NOT NULL
		REFERENCES parents(parent)
		ON DELETE CASCADE,
	preference TEXT NOT NULL
		REFERENCES parent_name_preference_values(value)
		ON DELETE CASCADE,
	PRIMARY KEY (parent, name)
);
