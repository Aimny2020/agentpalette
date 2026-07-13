CREATE TABLE IF NOT EXISTS project_harnesses (
  project_id TEXT PRIMARY KEY NOT NULL,
  source_template_id TEXT,
  source_template_hash TEXT,
  applied_at TEXT NOT NULL,
  managed_state TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS project_harness_files (
  project_id TEXT NOT NULL,
  path TEXT NOT NULL,
  applied_content_hash TEXT NOT NULL,
  created_by_application INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY (project_id, path),
  FOREIGN KEY (project_id) REFERENCES project_harnesses(project_id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO _migrations (version) VALUES (7);
