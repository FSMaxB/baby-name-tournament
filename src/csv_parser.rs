use serde::Deserialize;
use std::path::Path;

pub fn parse_csv(path: &Path) -> anyhow::Result<impl Iterator<Item = anyhow::Result<NameRecord>> + use<>> {
	// FIXME: Can I turn this ? into an  error result as part of the iterator itself?
	let reader = csv::Reader::from_path(path)?;
	Ok(reader.into_deserialize().map(|result| result.map_err(Into::into)))
}

#[derive(Clone, Debug, Deserialize)]
pub struct NameRecord {
	pub name: String,
	pub count: u32,
	pub gender: Gender,
}

#[derive(
	Clone, Copy, Debug, Deserialize, sqlx::Type, strum::Display, strum::EnumString, strum::AsRefStr, strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Gender {
	#[serde(alias = "w")]
	Female,
	#[serde(alias = "m")]
	Male,
	#[serde(alias = "n")]
	Both,
}
