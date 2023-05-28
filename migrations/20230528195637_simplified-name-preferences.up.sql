ALTER TABLE parent_name_preferences
	RENAME COLUMN mother_preference TO old_mother_preference;
ALTER TABLE parent_name_preferences
	RENAME COLUMN father_preference TO old_father_preference;

ALTER TABLE parent_name_preferences
	ADD COLUMN mother_preference TEXT
		REFERENCES parent_name_preference_values(value)
		ON UPDATE CASCADE
		ON DELETE RESTRICT;
ALTER TABLE parent_name_preferences
	ADD COLUMN father_preference
		REFERENCES parent_name_preference_values(value)
		ON UPDATE CASCADE
		ON DELETE RESTRICT;

UPDATE parent_name_preferences
SET
	mother_preference = nullif(old_mother_preference, 'neutral'),
	father_preference = nullif(old_father_preference, 'neutral');

ALTER TABLE parent_name_preferences
	DROP COLUMN old_mother_preference;
ALTER TABLE parent_name_preferences
	DROP COLUMN old_father_preference;

DELETE
FROM parent_name_preferences
WHERE
	(mother_preference IS NULL) AND (father_preference IS NULL);

DELETE
FROM parent_name_preference_values
WHERE
	value = 'neutral';
