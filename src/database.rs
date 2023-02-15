use crate::csv_parser::{Gender, NameRecord};
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
