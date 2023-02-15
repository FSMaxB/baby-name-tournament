CREATE TABLE name_records (
	name TEXT NOT NULL
		REFERENCES names(name),
	count INTEGER,
	gender TEXT NOT NULL,
	source TEXT NOT NULL,
	PRIMARY KEY (name, gender, source)
);

CREATE TABLE names (
	name TEXT NOT NULL PRIMARY KEY,
	gender TEXT NOT NULL
);
