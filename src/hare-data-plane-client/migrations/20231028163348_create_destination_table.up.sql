-- Create table destination
-- RDBMS: SQLite

CREATE TABLE IF NOT EXISTS "destination" (
    "uid" VARCHAR(40) NOT NULL PRIMARY KEY,
    "url" VARCHAR(2000) NOT NULL UNIQUE,
    "num_params" INTEGER NOT NULL,
    "is_fallback" BOOLEAN NOT NULL,
    "is_default_fallback" BOOLEAN NOT NULL,
    "description" TEXT NOT NULL
);

CREATE INDEX "destination_is_fallback" ON "destination" ("is_fallback");
CREATE INDEX "destination_is_default_fallback" ON "destination" ("is_default_fallback");
