use crate::csv_parser::parse_csv;
use clap::Parser;
use std::path::PathBuf;

mod csv_parser;

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
				tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
					for line in parse_csv(&name_list)? {
						let line = line?;
						println!("{line:?}");
					}
					Ok(())
				})
				.await??;
			}
		}
		Ok(())
	}
}
