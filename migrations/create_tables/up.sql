PRAGMA foreign_keys = ON;

CREATE TABLE exams (
    rowid INTEGER NOT NULL UNIQUE,
    code TEXT UNIQUE,
    name TEXT UNIQUE,
    PRIMARY KEY(rowid)
);

CREATE TABLE cnaef_areas (
    rowid INTEGER NOT NULL UNIQUE,
    code TEXT UNIQUE,
    name TEXT UNIQUE,
    PRIMARY KEY(rowid)
);

CREATE TABLE duration_units (
    rowid INTEGER NOT NULL UNIQUE,
    name TEXT UNIQUE,
    PRIMARY KEY(rowid)
);

CREATE TABLE durations (
    rowid INTEGER NOT NULL UNIQUE,
    unit INTEGER,
    ammount INTEGER,
    PRIMARY KEY(rowid),
    FOREIGN KEY(rowid) REFERENCES main(rowid) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(unit) REFERENCES duration_units(rowid)
);

CREATE TABLE institutions (
    rowid INTEGER NOT NULL UNIQUE,
    code TEXT UNIQUE,
    name TEXT UNIQUE,
    /* should be an array of lines - abstract as table */
    address TEXT UNIQUE,
    /* should be an array of numbers - abstract as table */
    phone_numbers TEXT UNIQUE,
    /* should be an array of email addresses - abstract as table */
    email_addresses TEXT UNIQUE,
    PRIMARY KEY(rowid)
);

CREATE TABLE mandatory_exams (
    rowid INTEGER NOT NULL UNIQUE,
    exam INTEGER UNIQUE,
    main INTEGER UNIQUE,
    PRIMARY KEY(rowid)
    FOREIGN KEY(exam) REFERENCES exams(rowid)/* DEFERRABLE INITIALLY DEFERRED*/,
    FOREIGN KEY(main) REFERENCES main(rowid)
);

CREATE TABLE main (
    rowid INTEGER NOT NULL UNIQUE,
    ects INTEGER,
    institution INTEGER,
    PRIMARY KEY(rowid),
    FOREIGN KEY(institution) REFERENCES institutions(rowid),
    FOREIGN KEY(rowid) REFERENCES durations(rowid) DEFERRABLE INITIALLY DEFERRED
);

CREATE VIEW expanded_main AS
SELECT main.ects, institutions.code as institution_code, institutions.name as institution_name, durations.ammount as duration_ammount, duration_units.name as duration_unit
FROM main
INNER JOIN institutions
ON institution = institutions.rowid
INNER JOIN duration_units
ON durations.unit = duration_units.rowid
INNER JOIN durations
ON durations.rowid = main.rowid;