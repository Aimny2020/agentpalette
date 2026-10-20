CREATE TABLE IF NOT EXISTS network_preferences (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  git_proxy_mode TEXT NOT NULL,
  git_proxy_url TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO _migrations (version) VALUES (9);
