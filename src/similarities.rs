use crate::utils::PrettyPrintedDuration;
use similar_string::{compare_similarity, lcs_length};
use std::time::Instant;
use stringmetrics::levenshtein;

#[derive(Debug)]
pub struct Similarity {
	pub a: String,
	pub b: String,
	pub levenshtein: u32,
	pub longest_common_substring: u16,
	#[expect(clippy::struct_field_names)]
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

	pub fn similar_enough_to_warrant_storing(&self) -> bool {
		self.levenshtein > 5 || self.longest_common_substring_similarity >= 0.5
	}
}

pub struct SimilarityStatistics {
	last_name_seen: Option<String>,
	total_pair_count: usize,
	stored_count: usize,
	name_count: usize,
	start_time: Instant,
}

impl Default for SimilarityStatistics {
	fn default() -> Self {
		Self {
			last_name_seen: None,
			total_pair_count: 0,
			stored_count: 0,
			name_count: 0,
			start_time: Instant::now(),
		}
	}
}

impl SimilarityStatistics {
	pub fn update_and_maybe_print(&mut self, name: &str, will_be_stored: bool) {
		self.total_pair_count += 1;
		self.stored_count += usize::from(will_be_stored);

		if Some(name) != self.last_name_seen.as_deref() {
			self.name_count += 1;
			self.last_name_seen = Some(name.to_owned());
			let elapsed = self.start_time.elapsed();
			println!(
				"{} pairs: {}\tnames: {}\tsimilarities stored: {}\t{name}",
				PrettyPrintedDuration::from(elapsed),
				self.total_pair_count,
				self.name_count,
				self.stored_count,
			);
		}
	}
}
