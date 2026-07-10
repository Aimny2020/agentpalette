CREATE TABLE IF NOT EXISTS harness_templates (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  work_type TEXT NOT NULL,
  source_type TEXT NOT NULL,
  source_path TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO _migrations (version) VALUES (5);
