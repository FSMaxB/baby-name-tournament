use similar_string::{compare_similarity, lcs_length};
use stringmetrics::levenshtein;

#[derive(Debug)]
pub struct Similarity {
	pub a: String,
	pub b: String,
	pub levenshtein: u32,
	pub longest_common_substring: u16,
	pub longest_common_substring_similarity: f64,
}

impl Similarity {
	pub fn calculate(a: String, b: String) -> Self {
		let levenshtein = levenshtein(&a, &b);
		let longest_common_substring = u16::try_from(lcs_length(&a, &b)).expect("LCS out of range of u16");
		let longest_common_substring_similarity = compare_similarity(&a, &b);
		Self {
			a,
			b,
			levenshtein,
			longest_common_substring,
			longest_common_substring_similarity,
		}
	}
}
