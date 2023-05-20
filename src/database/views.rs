use crate::csv_parser::Gender;
use crate::database::Name;
use sqlx::SqlitePool;

pub async fn read_similar_at_offset(
	name: &str,
	gender: Gender,
	threshold: f64,
	offset: i64,
	database_pool: &SqlitePool,
) -> sqlx::Result<Name> {
	sqlx::query_as!(
		Name,
		r#"
		SELECT
			name as "name!",
			gender as "gender!: Gender"
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

pub async fn read_name_at_offset(offset: u32, gender: Gender, database_pool: &SqlitePool) -> sqlx::Result<Name> {
	sqlx::query_as!(
		Name,
		r#"
		SELECT
			name as "name!",
			gender as "gender!: Gender"
		FROM names
		WHERE
			CASE $1
				WHEN 'both' THEN TRUE
				WHEN 'female' THEN gender != 'male'
				WHEN 'male' THEN gender != 'female'
			END
		ORDER BY name ASC
		LIMIT 1
		OFFSET $2
		"#,
		gender,
		offset,
	)
	.fetch_one(database_pool)
	.await
}

pub async fn count_names(gender: Gender, database_pool: &SqlitePool) -> sqlx::Result<i32> {
	sqlx::query_scalar!(
		r#"
		SELECT
			count(*) as "count!"
		FROM names
		WHERE
			CASE $1
				WHEN 'both' THEN TRUE
				WHEN 'female' THEN gender != 'male'
				WHEN 'male' THEN gender != 'female'
			END
		"#,
		gender,
	)
	.fetch_one(database_pool)
	.await
}
