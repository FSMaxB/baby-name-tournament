ALTER TABLE name_preference_values
	RENAME TO parent_name_preference_values;

CREATE TABLE parent_name_preferences
(
	name              TEXT NOT NULL PRIMARY KEY
		REFERENCES names (name)
			ON DELETE CASCADE,
	mother_preference TEXT
		REFERENCES parent_name_preference_values (value)
			ON UPDATE CASCADE ON DELETE RESTRICT,
	father_preference TEXT
		REFERENCES parent_name_preference_values (value)
			ON UPDATE CASCADE ON DELETE RESTRICT
);

INSERT INTO parent_name_preferences(name, mother_preference, father_preference)
SELECT name, preference, preference
FROM name_preference;

DROP TABLE name_preference;
