-- Your SQL goes here
CREATE TABLE cnaef_areas (
    rowid INTEGER NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY(rowid),
    UNIQUE(code),
    UNIQUE(name)
)