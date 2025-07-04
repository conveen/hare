-- Create table health_check
-- RDBMS: SQLite

CREATE TABLE IF NOT EXISTS "health_check" (
    "uid" VARCHAR(40) NOT NULL PRIMARY KEY,
    "check_field" BOOLEAN NOT NULL
)
