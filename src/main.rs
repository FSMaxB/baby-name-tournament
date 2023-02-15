use crate::csv_parser::NameRecord;
use clap::Parser;
use std::path::PathBuf;

mod csv_parser;

fn main() -> anyhow::Result<()> {
	let cli = Cli::parse();
	cli.run()
}

#[derive(Debug, Parser)]
struct Cli {
	name_list: PathBuf,
}

impl Cli {
	pub fn run(self) -> anyhow::Result<()> {
		let mut csv_reader = csv::Reader::from_path(self.name_list)?;

		for line in csv_reader.deserialize::<NameRecord>() {
			let line = line?;
			println!("{line:?}");
		}
		Ok(())
	}
}
