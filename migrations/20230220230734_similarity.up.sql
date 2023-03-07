CREATE TABLE similarities (
	a TEXT NOT NULL
		REFERENCES names(name)
		ON DELETE CASCADE,
	b TEXT NOT NULL
		REFERENCES names(name)
		ON DELETE CASCADE,
	levenshtein INT NOT NULL,
	longest_common_substring INT NOT NULL,
	longest_common_substring_similarity REAL NOT NULL,
	PRIMARY KEY (a, b)
)
