CREATE TABLE app_state (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE recent_projects (
  path           TEXT PRIMARY KEY,
  name           TEXT NOT NULL,
  last_opened_at TEXT NOT NULL
);

CREATE TABLE global_tools (
  tool_name       TEXT PRIMARY KEY,
  executable_path TEXT NOT NULL,
  version_string  TEXT,
  last_checked_at TEXT
);

INSERT INTO app_state (key, value) VALUES ('schema_version', '1');
