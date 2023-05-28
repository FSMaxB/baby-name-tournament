use crate::csv_parser::Gender;
use crate::database::NamePreference;
use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct NameWithPreferences {
	pub name: String,
	pub gender: Gender,
	pub mother_preference: NamePreference,
	pub father_preference: NamePreference,
}

pub async fn read_all_similar(
	name: &str,
	gender: Gender,
	threshold: f64,
	database_pool: &SqlitePool,
) -> sqlx::Result<Vec<NameWithPreferences>> {
	sqlx::query_as!(
		NameWithPreferences,
		r#"
		SELECT
			names.name as "name!",
			gender as "gender!: Gender",
			coalesce(parent_name_preferences.mother_preference, 'neutral') as "mother_preference!: NamePreference",
			coalesce(parent_name_preferences.father_preference, 'neutral') as "father_preference!: NamePreference"
		FROM similarities
		INNER JOIN names ON
			b = names.name
		LEFT JOIN parent_name_preferences
			ON names.name = parent_name_preferences.name
		WHERE
			(a = $1)
			AND (a != b)
			AND
				CASE $2
					WHEN 'both' THEN TRUE
					WHEN 'female' THEN gender != 'male'
					WHEN 'male' THEN gender != 'female'
				END
			AND (levenshtein < $3)
		ORDER BY b DESC
		"#,
		name,
		gender,
		threshold,
	)
	.fetch_all(database_pool)
	.await
}

pub async fn read_all_names(
	gender: Gender,
	include_favorite: bool,
	include_nogo: bool,
	include_neutral: bool,
	name_contains: Option<&str>,
	database_pool: &SqlitePool,
) -> sqlx::Result<Vec<NameWithPreferences>> {
	sqlx::query_as!(
		NameWithPreferences,
		r#"
		SELECT
			names.name as "name!",
			gender as "gender!: Gender",
			coalesce(parent_name_preferences.mother_preference, 'neutral') as "mother_preference!: NamePreference",
			coalesce(parent_name_preferences.father_preference, 'neutral') as "father_preference!: NamePreference"
		FROM names
		LEFT JOIN parent_name_preferences
			ON names.name = parent_name_preferences.name
		WHERE
			CASE $1
				WHEN 'both' THEN TRUE
				WHEN 'female' THEN gender != 'male'
				WHEN 'male' THEN gender != 'female'
			END
			AND (
				($2 AND (mother_preference = 'favorite' OR father_preference = 'favorite'))
				OR ($3 AND (mother_preference = 'no_go' OR father_preference = 'no_go'))
				OR ($4 AND (parent_name_preferences.name IS NULL OR mother_preference = 'neutral' OR father_preference = 'neutral'))
			)
			AND ($5 IS NULL OR (names.name LIKE ('%' || $5 || '%')))
		ORDER BY names.name ASC
		"#,
		gender,
		include_favorite,
		include_nogo,
		include_neutral,
		name_contains,
	)
	.fetch_all(database_pool)
	.await
}
