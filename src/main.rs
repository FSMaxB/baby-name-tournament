use crate::csv_parser::{parse_csv, Gender};
use crate::database::{
	initialize_database, insert_name_record, list_all, list_all_pairs, read_random, upsert_name, upsert_similarity,
};
use crate::similarities::Similarity;
use crate::tui::{run_tui, TuiMode};
use crate::utils::stream_blocking_iterator;
use anyhow::Context;
use clap::Parser;
use futures_util::{StreamExt, TryStreamExt};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use sqlx::SqlitePool;
use std::path::PathBuf;

mod csv_parser;
mod database;
mod similarities;
mod tui;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenvy::dotenv()?;

	let cli = Cli::parse();

	let database_pool = initialize_database(&cli.database_path).await?;

	cli.run(database_pool).await
}

#[derive(Debug, Parser)]
struct Cli {
	#[clap(long, env = "DATABASE_PATH")]
	database_path: PathBuf,
	#[clap(subcommand)]
	command: Command,
}

#[derive(Debug, Parser)]
enum Command {
	Parse { name_list: PathBuf },
	Ingest { name_list: PathBuf },
	ListAll { gender: Gender },
	Similarities,
	Random { gender: Gender },
	Tui,
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
			ListAll { gender } => {
				list_all(gender, &database_pool)
					.try_for_each(|name| {
						println!("{name:?}");
						std::future::ready(Ok(()))
					})
					.await?;
			}
			Similarities => {
				list_all_pairs(&database_pool)
					.try_for_each(|(a, b)| {
						let similarity = Similarity::calculate(a, b);
						println!("{similarity:?}");
						upsert_similarity(similarity, &database_pool)
					})
					.await?;
			}
			Random { gender } => {
				let name = read_random(gender, &database_pool).await?;
				println!("{name:?}")
			}
			Tui => {
				let _tui_mode = TuiMode::enable()?;

				let backend = CrosstermBackend::new(std::io::stdout());
				let terminal = Terminal::new(backend)?;

				run_tui(terminal).await?;
			}
		}
		Ok(())
	}
}
