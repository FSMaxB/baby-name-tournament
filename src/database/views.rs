use crate::csv_parser::Gender;
use crate::database::NamePreference;
use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct NameWithPreference {
	pub name: String,
	pub gender: Gender,
	pub preference: Option<NamePreference>,
}

pub async fn read_one(name: &str, database_pool: &SqlitePool) -> sqlx::Result<NameWithPreference> {
	sqlx::query_as!(
		NameWithPreference,
		r#"
		SELECT
			names.name as "name!",
			gender as "gender!: Gender",
			name_preference.preference as "preference?: NamePreference"
		FROM names
			LEFT JOIN name_preference ON
				names.name = name_preference.name
		WHERE
			names.name = $1
		"#,
		name,
	)
	.fetch_one(database_pool)
	.await
}

pub async fn read_all_names(
	gender: Gender,
	include_favorite: bool,
	include_nogo: bool,
	include_undecided: bool,
	name_contains: Option<&str>,
	database_pool: &SqlitePool,
) -> sqlx::Result<Vec<NameWithPreference>> {
	sqlx::query_as!(
		NameWithPreference,
		r#"
		SELECT
			names.name as "name!",
			gender as "gender!: Gender",
			name_preference.preference as "preference: NamePreference"
		FROM names
		LEFT JOIN name_preference
			ON names.name = name_preference.name
		WHERE
			CASE $1
				WHEN 'both' THEN TRUE
				WHEN 'female' THEN gender != 'male'
				WHEN 'male' THEN gender != 'female'
			END
			AND (
				($2 AND preference = 'favorite')
				OR ($3 AND preference = 'no_go')
				OR ($4 AND preference IS NULL)
			)
			AND ($5 IS NULL OR (names.name LIKE ('%' || $5 || '%')))
		ORDER BY names.name ASC
		"#,
		gender,
		include_favorite,
		include_nogo,
		include_undecided,
		name_contains,
	)
	.fetch_all(database_pool)
	.await
}
