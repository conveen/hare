-- Create table alias
-- RDBMS: SQLite

CREATE TABLE IF NOT EXISTS "alias" (
    "name" VARCHAR(100) NOT NULL PRIMARY KEY,
    "destination_uid" VARCHAR(40) NOT NULL REFERENCES "destination" ("uid") ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED 
);

CREATE INDEX "alias_destination_id" ON "alias" ("destination_uid");
