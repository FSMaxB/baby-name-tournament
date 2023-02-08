use clap::Parser;
use serde::Deserialize;
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
		let mut csv_reader = csv::Reader::from_path(self.name_list)?;

		for line in csv_reader.deserialize::<(String, usize, Gender)>() {
			let line = line?;
			println!("{line:?}");
		}
		Ok(())
	}
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Gender {
	#[serde(alias = "w")]
	Female,
	#[serde(alias = "m")]
	Male,
	#[serde(alias = "n")]
	Both,
}
