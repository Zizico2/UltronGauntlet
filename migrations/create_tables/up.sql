PRAGMA foreign_keys = ON;

CREATE TABLE exams (
    code TEXT NOT NULL UNIQUE,
    name TEXT UNIQUE,
    PRIMARY KEY(code)
);

CREATE TABLE cnaef_areas (
    code TEXT NOT NULL UNIQUE,
    name TEXT UNIQUE,
    PRIMARY KEY(code)
);

CREATE TABLE degrees (
    name TEXT NOT NULL UNIQUE, /* Licenciatura -     1º ciclo */
    PRIMARY KEY(name)
);

CREATE TABLE education_types (
    name TEXT NOT NULL UNIQUE, /* Universitário */
    PRIMARY KEY(name)
);

CREATE TABLE contests (
    name TEXT NOT NULL UNIQUE, /* Nacional */
    PRIMARY KEY(name)
);

/* Some tables to handle prerequisites */

CREATE TABLE duration_units (
    name TEXT NOT NULL UNIQUE,
    PRIMARY KEY(name)
);

CREATE TABLE durations (
    /**/
    institution TEXT NOT NULL,
    course TEXT NOT NULL,
    /**/
    unit TEXT,
    ammount INTEGER,
    UNIQUE(institution, course),
    PRIMARY KEY(institution, course),
    FOREIGN KEY(institution, course) REFERENCES course_institution(institution, course) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(unit) REFERENCES duration_units(name)
);

CREATE TABLE institutions (
    code TEXT NOT NULL UNIQUE,
    name TEXT UNIQUE,
    /* should be an array of lines - abstract as table */
    address TEXT UNIQUE,
    /* should be an array of numbers - abstract as table */
    phone_numbers TEXT UNIQUE,
    /* should be an array of email addresses - abstract as table */
    email_addresses TEXT UNIQUE,
    PRIMARY KEY(code)
);

/* rowid pk? */
CREATE TABLE mandatory_exams (
    rowid INTEGER NOT NULL UNIQUE,
    exam TEXT UNIQUE,
    /**/
    institution TEXT NOT NULL,
    course TEXT NOT NULL,
    /**/
    PRIMARY KEY(rowid)
    FOREIGN KEY(exam) REFERENCES exams(code),
    UNIQUE(institution, course),
    FOREIGN KEY(institution, course) REFERENCES course_institution(institution, course)
);

CREATE TABLE courses (
    code TEXT NOT NULL UNIQUE,
    name TEXT UNIQUE,
    PRIMARY KEY(code)
);

CREATE TABLE course_institution (
    ects INTEGER,
    institution TEXT NOT NULL,
    course TEXT NOT NULL,
    UNIQUE(institution, course),
    PRIMARY KEY(institution, course),
    FOREIGN KEY(institution) REFERENCES institutions(code),
    FOREIGN KEY(course) REFERENCES courses(code),
    FOREIGN KEY(institution, course) REFERENCES durations(institution, course) DEFERRABLE INITIALLY DEFERRED
);

CREATE VIEW expanded_course_institution AS
SELECT course_institution.ects,
institutions.code as institution_code,
institutions.name as institution_name,

durations.ammount as duration_ammount,

duration_units.name as duration_unit


FROM course_institution
INNER JOIN institutions
ON course_institution.institution = institutions.code
INNER JOIN duration_units
ON durations.unit = duration_units.name
INNER JOIN durations
ON durations.rowid = course_institution.rowid;