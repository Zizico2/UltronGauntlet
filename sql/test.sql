-- SQLite
INSERT INTO institutions (code, name, address, phone_numbers, email_addresses)
VALUES ("INST1", "INSTNAME1", "ADDR1", "PHONE1", "EMAIL1");

BEGIN TRANSACTION;
INSERT INTO duration_units (name)
VALUES ("Semestres");

INSERT INTO durations (unit, ammount)
VALUES (last_insert_rowid(), 6);

INSERT INTO main (rowid, ects, institution)
VALUES (last_insert_rowid(), 120, 1);
COMMIT TRANSACTION;