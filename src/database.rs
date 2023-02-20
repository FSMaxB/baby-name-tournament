use crate::csv_parser::{Gender, NameRecord};
use futures_util::{stream, Stream, TryStreamExt};
use sqlx::SqlitePool;
use url::Url;

pub async fn initialize_database(url: &Url) -> anyhow::Result<SqlitePool> {
	let pool = SqlitePool::connect(url.as_str()).await?;

	sqlx::migrate!("./migrations").run(&pool).await?;

	Ok(pool)
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
