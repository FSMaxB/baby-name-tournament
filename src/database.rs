use crate::csv_parser::{Gender, NameRecord};
use futures_util::{Stream, TryStreamExt, stream};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
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
		.auto_vacuum(SqliteAutoVacuum::Incremental)
		.pragma("mmap_size", (32usize * 1024 * 1024 * 1024).to_string())
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type, strum::AsRefStr)]
#[sqlx(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NamePreference {
	Favorite,
	NoGo,
}

pub fn list_all(gender: Gender, database_pool: &SqlitePool) -> impl Stream<Item = sqlx::Result<Name>> + use<> {
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

pub async fn upsert_name_preference(
	name: &str,
	preference: NamePreference,
	database_pool: &SqlitePool,
) -> sqlx::Result<()> {
	sqlx::query!(
		r#"
		INSERT INTO name_preference (
			name,
			preference
		) VALUES ($1, $2)
		ON CONFLICT DO UPDATE
		SET
			preference = $2
		"#,
		name,
		preference,
	)
	.execute(database_pool)
	.await?;
	Ok(())
}

pub async fn delete_name_preference(name: &str, database_pool: &SqlitePool) -> sqlx::Result<()> {
	sqlx::query!(
		r#"
		DELETE FROM name_preference
		WHERE name = $1
		"#,
		name,
	)
	.execute(database_pool)
	.await?;
	Ok(())
}

pub mod views;
