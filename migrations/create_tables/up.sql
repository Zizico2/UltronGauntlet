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
    UNIQUE(rowid),
    PRIMARY KEY(rowid),
    FOREIGN KEY(rowid) REFERENCES main(rowid) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(unit) REFERENCES duration_units(rowid) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE institutions (
    rowid INTEGER NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    /* should be an array of lines - abstract as table */
    address TEXT NOT NULL,
    /* should be an array of numbers - abstract as table */
    phone_numbers TEXT NOT NULL,
    /* should be an array of email addresses - abstract as table */
    email_addresses TEXT NOT NULL,
    UNIQUE(rowid),
    PRIMARY KEY(rowid),
    UNIQUE(code),
    FOREIGN KEY(rowid) REFERENCES main(rowid) DEFERRABLE INITIALLY DEFERRED
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
    institution INTEGER NOT NULL,
    UNIQUE(rowid),
    PRIMARY KEY(rowid),
    FOREIGN KEY(institution) REFERENCES institutions(rowid) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(rowid) REFERENCES durations(rowid) DEFERRABLE INITIALLY DEFERRED
);