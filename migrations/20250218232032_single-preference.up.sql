ALTER TABLE parent_name_preference_values
	RENAME TO name_preference_values;

CREATE TABLE name_preference
(
	name       TEXT NOT NULL PRIMARY KEY
		REFERENCES names (name)
			ON DELETE CASCADE,
	preference TEXT NOT NULL
		REFERENCES name_preference_values (value)
			ON UPDATE CASCADE ON DELETE RESTRICT
);

INSERT INTO name_preference(name, preference)
SELECT name,
	   CASE mother_preference
		   WHEN 'favorite' THEN 'favorite'
		   WHEN 'no_go' THEN
			   CASE father_preference
				   WHEN 'favorite' THEN 'favorite'
				   ELSE 'no_go'
				   END
		   ELSE father_preference
		   END as preference
FROM parent_name_preferences
WHERE mother_preference IS NOT NULL
   OR father_preference IS NOT NULL;

DROP TABLE parent_name_preferences;
