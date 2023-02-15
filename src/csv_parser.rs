use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct NameRecord {
	pub name: String,
	pub count: usize,
	pub gender: Gender,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
	#[serde(alias = "w")]
	Female,
	#[serde(alias = "m")]
	Male,
	#[serde(alias = "n")]
	Both,
}
