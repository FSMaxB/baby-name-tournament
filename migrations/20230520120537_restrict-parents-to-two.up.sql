DROP TABLE parent_name_preferences;
DROP TABLE parents;

INSERT INTO parent_name_preference_values(value) VALUES ('neutral');

CREATE TABLE parent_name_preferences (
	name TEXT PRIMARY KEY
		REFERENCES names(name)
		ON DELETE CASCADE,
	mother_preference TEXT NOT NULL DEFAULT 'neutral'
		REFERENCES parent_name_preference_values(value)
		ON DELETE CASCADE,
	father_preference TEXT NOT NULL DEFAULT 'neutral'
		REFERENCES parent_name_preference_values(value)
			ON DELETE CASCADE
);
