use crate::csv_parser::parse_csv;
use crate::database::{initialize_database, insert_name_record, upsert_name};
use crate::utils::stream_blocking_iterator;
use anyhow::Context;
use clap::Parser;
use futures_util::StreamExt;
use sqlx::SqlitePool;
use std::path::PathBuf;
use url::Url;

mod csv_parser;
mod database;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenvy::dotenv()?;

	let cli = Cli::parse();

	let database_pool = initialize_database(&cli.database_url).await?;

	cli.run(database_pool).await
}

#[derive(Debug, Parser)]
struct Cli {
	#[clap(long, env = "DATABASE_URL")]
	database_url: Url,
	#[clap(subcommand)]
	command: Command,
}

#[derive(Debug, Parser)]
enum Command {
	Parse { name_list: PathBuf },
	Ingest { name_list: PathBuf },
}

impl Cli {
	pub async fn run(self, database_pool: SqlitePool) -> anyhow::Result<()> {
		use Command::*;
		match self.command {
			Parse { name_list } => {
				let mut name_stream = stream_blocking_iterator(parse_csv(&name_list)?);
				while let Some(line) = name_stream.next().await {
					let line = line?;
					println!("{line:?}");
				}
			}
			Ingest { name_list } => {
				let source = name_list.file_name().context("Missing filename")?.to_string_lossy();

				let mut name_stream = stream_blocking_iterator(parse_csv(&name_list)?);
				while let Some(record) = name_stream.next().await {
					let record = record?;
					upsert_name(&record.name, record.gender, &database_pool).await?;
					insert_name_record(&record, &source, &database_pool).await?;
				}
			}
		}
		Ok(())
	}
}
