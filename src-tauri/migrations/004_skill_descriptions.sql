CREATE TABLE IF NOT EXISTS skill_descriptions (
    target_id TEXT PRIMARY KEY NOT NULL,
    target_kind TEXT NOT NULL CHECK (target_kind IN ('package', 'member')),
    custom_description TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO _migrations (version) VALUES (4);
