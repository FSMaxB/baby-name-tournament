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

pub async fn read_similar_at_offset(
	name: &str,
	gender: Gender,
	threshold: f64,
	offset: i64,
	database_pool: &SqlitePool,
) -> sqlx::Result<NameWithPreferences> {
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
		LIMIT -1
		OFFSET $4
		"#,
		name,
		gender,
		threshold,
		offset
	)
	.fetch_one(database_pool)
	.await
}

pub async fn count_similar(
	name: &str,
	gender: Gender,
	threshold: f64,
	database_pool: &SqlitePool,
) -> sqlx::Result<i32> {
	sqlx::query_scalar!(
		r#"
		SELECT
			count(*)
		FROM similarities
		INNER JOIN names ON
			b = name
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
		"#,
		name,
		gender,
		threshold,
	)
	.fetch_optional(database_pool)
	.await
	.map(Option::unwrap_or_default)
}

pub async fn read_name_at_offset(
	offset: u32,
	gender: Gender,
	include_favorite: bool,
	include_nogo: bool,
	include_neutral: bool,
	database_pool: &SqlitePool,
) -> sqlx::Result<NameWithPreferences> {
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
				OR ($3 AND (mother_preference = 'nogo' OR father_preference = 'nogo'))
				OR ($4 AND (parent_name_preferences.name IS NULL OR mother_preference = 'neutral' OR father_preference = 'neutral'))
			)
		ORDER BY names.name ASC
		LIMIT 1
		OFFSET $5
		"#,
		gender,
		include_favorite,
		include_nogo,
		include_neutral,
		offset,
	)
	.fetch_one(database_pool)
	.await
}

pub async fn count_names(
	gender: Gender,
	include_favorite: bool,
	include_nogo: bool,
	include_neutral: bool,
	database_pool: &SqlitePool,
) -> sqlx::Result<i32> {
	sqlx::query_scalar!(
		r#"
		SELECT
			count(*) as "count!"
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
				OR ($3 AND (mother_preference = 'nogo' OR father_preference = 'nogo'))
				OR ($4 AND (parent_name_preferences.name IS NULL OR mother_preference = 'neutral' OR father_preference = 'neutral'))
			)
		"#,
		gender,
		include_favorite,
		include_nogo,
		include_neutral,
	)
	.fetch_one(database_pool)
	.await
}
