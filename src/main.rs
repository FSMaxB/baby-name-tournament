use crate::csv_parser::{parse_csv, Gender};
use crate::similarities::Similarity;
use crate::tui::{run_tui, TuiMode};
use crate::utils::stream_blocking_iterator;
use anyhow::Context;
use clap::Parser;
use futures_util::{StreamExt, TryStreamExt};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tokio::runtime;

mod csv_parser;
mod database;
mod similarities;
mod tui;
mod utils;

fn main() -> anyhow::Result<()> {
	dotenvy::dotenv()?;

	let cli = Cli::parse();

	cli.run()
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
	pub fn run(self) -> anyhow::Result<()> {
		use Command::*;

		let runtime = runtime::Builder::new_current_thread().enable_all().build()?;

		let database_pool = runtime.block_on(database::initialize(&self.database_path))?;

		match self.command {
			Parse { name_list } => {
				runtime.block_on(parse(&name_list))?;
			}
			Ingest { name_list } => {
				runtime.block_on(ingest(&name_list, database_pool))?;
			}
			ListAll { gender } => {
				runtime.block_on(list_all(gender, database_pool))?;
			}
			Similarities => {
				runtime.block_on(similarities(database_pool))?;
			}
			Random { gender } => {
				runtime.block_on(random(gender, database_pool))?;
			}
			Tui => {
				runtime.block_on(tui(database_pool))?;
			}
		}
		Ok(())
	}
}

pub async fn parse(name_list: &Path) -> anyhow::Result<()> {
	let mut name_stream = stream_blocking_iterator(parse_csv(&name_list)?);
	while let Some(line) = name_stream.next().await {
		let line = line?;
		println!("{line:?}");
	}
	Ok(())
}

pub async fn ingest(name_list: &Path, database_pool: SqlitePool) -> anyhow::Result<()> {
	let source = name_list.file_name().context("Missing filename")?.to_string_lossy();

	let mut name_stream = stream_blocking_iterator(parse_csv(&name_list)?);
	while let Some(record) = name_stream.next().await {
		let record = record?;
		database::upsert_name(&record.name, record.gender, &database_pool).await?;
		database::insert_name_record(&record, &source, &database_pool).await?;
	}
	Ok(())
}

pub async fn list_all(gender: Gender, database_pool: SqlitePool) -> anyhow::Result<()> {
	database::list_all(gender, &database_pool)
		.try_for_each(|name| {
			println!("{name:?}");
			std::future::ready(Ok(()))
		})
		.await?;
	Ok(())
}

pub async fn similarities(database_pool: SqlitePool) -> anyhow::Result<()> {
	database::list_all_pairs(&database_pool)
		.try_for_each(|(a, b)| {
			let similarity = Similarity::calculate(a, b);
			println!("{similarity:?}");
			database::upsert_similarity(similarity, &database_pool)
		})
		.await?;
	Ok(())
}

pub async fn random(gender: Gender, database_pool: SqlitePool) -> anyhow::Result<()> {
	let name = database::read_random(gender, &database_pool).await?;
	println!("{name:?}");
	Ok(())
}

pub async fn tui(database_pool: SqlitePool) -> anyhow::Result<()> {
	let _tui_mode = TuiMode::enable()?;

	let backend = CrosstermBackend::new(std::io::stdout());
	let terminal = Terminal::new(backend)?;

	run_tui(database_pool, terminal).await?;
	Ok(())
}
