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
    name TEXT NOT NULL,
    PRIMARY KEY(rowid),
    UNIQUE(rowid),
    UNIQUE(name)
);

CREATE TABLE durations (
    rowid INTEGER NOT NULL,
    unit INTEGER NOT NULL,
    ammount INTEGER NOT NULL,
    PRIMARY KEY(rowid),
    FOREIGN KEY(rowid) REFERENCES main(rowid) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(unit) REFERENCES duration_units(rowid) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE mandatory_exams (
    rowid INTEGER NOT NULL,
    exam INTEGER NOT NULL,
    main INTEGER NOT NULL,
    PRIMARY KEY(rowid),
    UNIQUE(rowid),
    FOREIGN KEY(exam) REFERENCES exams(rowid) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(main) REFERENCES main(rowid) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE main (
    rowid INTEGER NOT NULL,
    ects INTEGER NOT NULL,
    PRIMARY KEY(rowid),
    FOREIGN KEY(rowid) REFERENCES durations(rowid) DEFERRABLE INITIALLY DEFERRED,
    UNIQUE(rowid)
);