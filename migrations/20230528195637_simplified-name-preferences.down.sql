INSERT INTO parent_name_preference_values(value) VALUES ('neutral');

ALTER TABLE parent_name_preferences RENAME TO old_parent_name_preferences;

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

INSERT INTO parent_name_preferences (
	name,
	mother_preference,
	father_preference
) SELECT
	name,
	coalesce(mother_preference, 'neutral'),
	coalesce(father_preference, 'neutral')
FROM old_parent_name_preferences;

DROP TABLE old_parent_name_preferences;
