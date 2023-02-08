use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

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
		let name_list = File::open(self.name_list)?;

		for line in BufReader::new(name_list).lines() {
			let line = line?;
			println!("{line}");
		}
		Ok(())
	}
}
