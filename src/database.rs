use crate::csv_parser::{Gender, NameRecord};
use crate::similarities::Similarity;
use futures_util::{stream, Stream, TryStreamExt};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::SqlitePool;
use std::path::Path;

pub async fn initialize(path: &Path) -> anyhow::Result<SqlitePool> {
	let pool = SqlitePool::connect_with(connect_options(path)).await?;

	sqlx::migrate!("./migrations").run(&pool).await?;

	Ok(pool)
}

fn connect_options(path: &Path) -> SqliteConnectOptions {
	SqliteConnectOptions::new()
		.filename(path)
		.journal_mode(SqliteJournalMode::Wal)
		.synchronous(SqliteSynchronous::Normal)
		.create_if_missing(true)
}

pub async fn upsert_name(name: &str, gender: Gender, database_pool: &SqlitePool) -> anyhow::Result<()> {
	sqlx::query!(
		r#"
		INSERT INTO names (
			name,
			gender
		) VALUES ($1, $2)
		ON CONFLICT DO
		UPDATE
		SET
			gender = CASE names.gender
				WHEN 'female' THEN
					CASE $2
						WHEN 'female' THEN 'female'
						ELSE 'both'
					END
				WHEN 'male' THEN
					CASE $2
						WHEN 'male' THEN 'male'
						ELSE 'both'
					END
				WHEN 'both' THEN 'both'
			END
		"#,
		name,
		gender,
	)
	.execute(database_pool)
	.await?;
	Ok(())
}

pub async fn upsert_similarity(
	Similarity {
		a,
		b,
		levenshtein,
		longest_common_substring,
		longest_common_substring_similarity,
	}: Similarity,
	database_pool: &SqlitePool,
) -> sqlx::Result<()> {
	sqlx::query!(
		r#"
		INSERT INTO similarities (
			a,
			b,
			levenshtein,
			longest_common_substring,
			longest_common_substring_similarity
		) VALUES ($1, $2, $3, $4, $5)
		ON CONFLICT DO
		UPDATE
		SET
			levenshtein = $3,
			longest_common_substring = $4,
			longest_common_substring_similarity = $5
		"#,
		a,
		b,
		levenshtein,
		longest_common_substring,
		longest_common_substring_similarity,
	)
	.execute(database_pool)
	.await?;
	Ok(())
}

pub async fn insert_name_record(
	NameRecord { name, count, gender }: &NameRecord,
	source: &str,
	database_pool: &SqlitePool,
) -> sqlx::Result<()> {
	sqlx::query!(
		r#"
		INSERT INTO name_records (
			name,
			count,
			gender,
			source
		) VALUES ($1, $2, $3, $4)
		ON CONFLICT DO NOTHING
		"#,
		name,
		count,
		gender,
		source
	)
	.execute(database_pool)
	.await?;
	Ok(())
}

#[derive(Debug, Clone)]
pub struct Name {
	pub name: String,
	pub gender: Gender,
}

pub fn list_all(gender: Gender, database_pool: &SqlitePool) -> impl Stream<Item = sqlx::Result<Name>> {
	const BULK_SIZE: i64 = 10;
	let database_pool = database_pool.clone();
	stream::try_unfold(String::new(), move |names_after| {
		let database_pool = database_pool.clone();
		async move {
			let names = sqlx::query_as!(
				Name,
				r#"
			SELECT
				name,
				gender as "gender: Gender"
			FROM names
			WHERE
				name > $1
				AND CASE $2
					WHEN 'both' THEN TRUE
					WHEN 'female' THEN gender != 'male'
					WHEN 'male' THEN gender != 'female'
				END
			ORDER BY name ASC
			LIMIT $3
			"#,
				names_after,
				gender,
				BULK_SIZE,
			)
			.fetch_all(&database_pool)
			.await?;
			Ok::<_, sqlx::Error>(names.last().cloned().map(|Name { name, .. }| (names, name)))
		}
	})
	.map_ok(|names: Vec<Name>| stream::iter(names.into_iter().map(Ok)))
	.try_flatten()
}

pub fn list_all_pairs(database_pool: &SqlitePool) -> impl Stream<Item = sqlx::Result<(String, String)>> {
	#[derive(Clone, Default)]
	struct Pair {
		a: String,
		b: String,
	}

	const BULK_SIZE: i64 = 1000;
	let database_pool = database_pool.clone();
	stream::try_unfold(Pair::default(), move |Pair { a: last_a, b: last_b }| {
		let database_pool = database_pool.clone();
		async move {
			let pairs = sqlx::query_as!(
				Pair,
				r#"
				SELECT
					a.name as a,
					b.name as b
				FROM names as a
					CROSS JOIN names as b
				WHERE
					(a.name, b.name) > ($1, $2)
				ORDER BY a.name, b.name
				LIMIT $3
			"#,
				last_a,
				last_b,
				BULK_SIZE,
			)
			.fetch_all(&database_pool)
			.await?;
			Ok::<_, sqlx::Error>(pairs.last().cloned().map(|pair| (pairs, pair)))
		}
	})
	.map_ok(|pairs: Vec<Pair>| stream::iter(pairs.into_iter().map(|Pair { a, b }| Ok((a, b)))))
	.try_flatten()
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

pub async fn read_random(gender: Gender, database_pool: &SqlitePool) -> sqlx::Result<Name> {
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
		ORDER BY RANDOM()
		LIMIT 1
		"#,
		gender,
	)
	.fetch_one(database_pool)
	.await
}
