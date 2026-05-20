CREATE TABLE projects (
  id                  INTEGER PRIMARY KEY,
  name                TEXT NOT NULL,
  description         TEXT,
  created_at          TEXT NOT NULL,
  app_schema_version  INTEGER NOT NULL
);
