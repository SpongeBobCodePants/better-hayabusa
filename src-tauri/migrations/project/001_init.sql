CREATE TABLE projects (
  id                  INTEGER PRIMARY KEY,
  name                TEXT NOT NULL,
  description         TEXT CHECK (description IS NULL OR length(description) <= 250),
  created_at          TEXT NOT NULL,
  app_schema_version  INTEGER NOT NULL
);
