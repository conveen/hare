-- Create table health_check
-- RDBMS: SQLite

CREATE TABLE IF NOT EXISTS "health_check" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "check_field" BOOLEAN NOT NULL
)
