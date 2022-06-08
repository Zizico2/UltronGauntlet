CREATE TABLE exams (
    rowid INTEGER NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY(rowid),
    UNIQUE(rowid),
    UNIQUE(code),
    UNIQUE(name)
);

CREATE TABLE cnaef_areas (
    rowid INTEGER NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY(rowid),
    UNIQUE(rowid),
    UNIQUE(code),
    UNIQUE(name)
);

CREATE TABLE duration_units (
    rowid INTEGER NOT NULL,
    unit TEXT NOT NULL,
    PRIMARY KEY(rowid),
    UNIQUE(rowid),
    UNIQUE(unit)
);

CREATE TABLE mandatory_exams (
    rowid INTEGER NOT NULL,
    exam INTEGER NOT NULL,
    main INTEGER NOT NULL,
    PRIMARY KEY(rowid),
    UNIQUE(rowid),
    FOREIGN KEY(exam) REFERENCES exams(rowid),
    FOREIGN KEY(main) REFERENCES main(rowid)
);

CREATE TABLE main (
    rowid INTEGER NOT NULL,
    ects INTEGER NOT NULL,
    PRIMARY KEY(rowid),
    UNIQUE(rowid)
);