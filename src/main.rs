use crate::csv_parser::{Gender, parse_csv};
use crate::utils::stream_blocking_iterator;
use anyhow::Context;
use clap::Parser;
use futures_util::TryStreamExt;
use sqlx::SqlitePool;
use std::future;
use std::path::{Path, PathBuf};
use tokio::runtime;

mod csv_parser;
mod database;
mod gtk;
mod utils;

fn main() -> anyhow::Result<()> {
	if let Err(error) = dotenvy::dotenv() {
		println!("Error loading .env file: {error}");
	}

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
	Random { gender: Gender },
	GtkGui,
}

impl Cli {
	pub fn run(self) -> anyhow::Result<()> {
		use Command::*;

		let runtime = runtime::Builder::new_current_thread().enable_all().build()?;
		let database_pool = runtime.block_on(database::initialize(&self.database_path))?;

		match self.command {
			Parse { name_list } => {
				runtime.block_on(parse(&name_list))?;
				runtime.block_on(database_pool.close());
			}
			Ingest { name_list } => {
				runtime.block_on(ingest(&name_list, database_pool.clone()))?;
				runtime.block_on(database_pool.close());
			}
			ListAll { gender } => {
				runtime.block_on(list_all(gender, database_pool.clone()))?;
				runtime.block_on(database_pool.close());
			}
			Random { gender } => {
				runtime.block_on(random(gender, database_pool.clone()))?;
				runtime.block_on(database_pool.close());
			}
			GtkGui => {
				gtk::start(runtime, &database_pool)?;
			}
		}
		Ok(())
	}
}

pub async fn parse(name_list: &Path) -> anyhow::Result<()> {
	stream_blocking_iterator(parse_csv(name_list)?)
		.try_for_each(|line| {
			println!("{line:?}");
			future::ready(Ok(()))
		})
		.await?;

	Ok(())
}

pub async fn ingest(name_list: &Path, database_pool: SqlitePool) -> anyhow::Result<()> {
	let source = name_list.file_name().context("Missing filename")?.to_string_lossy();

	stream_blocking_iterator(parse_csv(name_list)?)
		.try_for_each(|record| {
			let database_pool = &database_pool;
			let source = &source;
			async move {
				database::upsert_name(&record.name, record.gender, database_pool).await?;
				Ok(database::insert_name_record(&record, source, database_pool).await?)
			}
		})
		.await?;
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

pub async fn random(gender: Gender, database_pool: SqlitePool) -> anyhow::Result<()> {
	let name = database::read_random(gender, &database_pool).await?;
	println!("{name:?}");
	Ok(())
}
