-- SQLite
PRAGMA foreign_keys = ON;

INSERT INTO institutions (code, name, address, phone_numbers, email_addresses)
VALUES ("INST1", "INSTNAME1", "ADDR1", "PHONE1", "EMAIL1");

INSERT INTO courses (code, name)
VALUES ("COURSE1", "COURSENAME1");

BEGIN TRANSACTION;
INSERT INTO duration_units (name)
VALUES ("Semestres");

INSERT INTO durations (institution, course, unit, ammount)
VALUES ("INST1", "COURSE1", "Semestres", 6);

INSERT INTO course_institution (institution, course, ects, institution)
VALUES ("INST1", "COURSE1", 120, 1);
COMMIT TRANSACTION;