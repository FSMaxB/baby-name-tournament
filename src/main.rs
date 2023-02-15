use crate::csv_parser::parse_csv;
use crate::utils::stream_blocking_iterator;
use clap::Parser;
use futures_util::StreamExt;
use std::path::PathBuf;

mod csv_parser;
mod utils;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	let cli = Cli::parse();

	cli.run().await
}

#[derive(Debug, Parser)]
struct Cli {
	#[clap(subcommand)]
	command: Command,
}

#[derive(Debug, Parser)]
enum Command {
	Print { name_list: PathBuf },
}

impl Cli {
	pub async fn run(self) -> anyhow::Result<()> {
		use Command::*;
		match self.command {
			Print { name_list } => {
				let mut name_stream = stream_blocking_iterator(parse_csv(&name_list)?);
				while let Some(line) = name_stream.next().await {
					let line = line?;
					println!("{line:?}");
				}
			}
		}
		Ok(())
	}
}
