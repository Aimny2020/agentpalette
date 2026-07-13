use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use rusqlite::OptionalExtension;

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::health::{DatabasePort, DatabaseStatus};
use crate::domain::ports::{HarnessRepository, SkillRepository};
use crate::domain::project::Project;
use crate::domain::skill::{Category, SkillPackageRecord, SourceKind, UserSkillMeta};

const INITIAL_MIGRATION: &str = include_str!("../../migrations/001_initial.sql");
const SKILLS_MIGRATION: &str = include_str!("../../migrations/002_skills.sql");
const SKILL_PACKAGES_MIGRATION: &str = include_str!("../../migrations/003_skill_packages.sql");
const SKILL_DESCRIPTIONS_MIGRATION: &str =
    include_str!("../../migrations/004_skill_descriptions.sql");
const HARNESSES_MIGRATION: &str = include_str!("../../migrations/005_harness_templates.sql");
const HARNESS_PRESET_MIGRATION: &str =
    include_str!("../../migrations/006_harness_template_preset.sql");

pub struct SqliteDatabase {
    connection: Mutex<Connection>,
}

impl SqliteDatabase {
    pub fn open(path: &Path) -> DomainResult<Self> {
        let connection = Connection::open(path).map_err(database_error)?;
        Self::initialize(connection)
    }

    pub fn open_in_memory() -> DomainResult<Self> {
        let connection = Connection::open_in_memory().map_err(database_error)?;
        Self::initialize(connection)
    }

    fn initialize(connection: Connection) -> DomainResult<Self> {
        connection
            .execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(database_error)?;
        connection
            .execute_batch(INITIAL_MIGRATION)
            .map_err(database_error)?;
        connection
            .execute_batch(SKILLS_MIGRATION)
            .map_err(database_error)?;
        connection
            .execute_batch(SKILL_PACKAGES_MIGRATION)
            .map_err(database_error)?;
        connection
            .execute_batch(SKILL_DESCRIPTIONS_MIGRATION)
            .map_err(database_error)?;
        connection
            .execute_batch(HARNESSES_MIGRATION)
            .map_err(database_error)?;
        let has_preset_column = {
            let mut statement = connection
                .prepare("PRAGMA table_info(harness_templates)")
                .map_err(database_error)?;
            let columns = statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(database_error)?;
            columns
                .collect::<Result<Vec<_>, _>>()
                .map_err(database_error)?
                .iter()
                .any(|column| column == "created_from_preset")
        };
        if !has_preset_column {
            connection
                .execute_batch(HARNESS_PRESET_MIGRATION)
                .map_err(database_error)?;
        } else {
            connection
                .execute("INSERT OR IGNORE INTO _migrations (version) VALUES (6)", [])
                .map_err(database_error)?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    #[cfg(test)]
    fn has_table(&self, name: &str) -> DomainResult<bool> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let found = connection
            .query_row(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [name],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(database_error)?;
        Ok(found.is_some())
    }
}

impl DatabasePort for SqliteDatabase {
    fn status(&self) -> DomainResult<DatabaseStatus> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .query_row("SELECT 1", [], |_| Ok(()))
            .map_err(database_error)?;
        Ok(DatabaseStatus::Ready)
    }
}

impl SkillRepository for SqliteDatabase {
    fn get_projects(&self) -> DomainResult<Vec<Project>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let mut stmt = connection
            .prepare("SELECT id, name, path, created_at FROM projects ORDER BY created_at ASC")
            .map_err(database_error)?;

        let iter = stmt
            .query_map([], |r| {
                Ok(Project {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    path: r.get(2)?,
                    created_at: r.get(3)?,
                })
            })
            .map_err(database_error)?;

        let mut list = Vec::new();
        for item in iter {
            list.push(item.map_err(database_error)?);
        }
        Ok(list)
    }

    fn get_project_path(&self, id: &str) -> DomainResult<Option<String>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let mut stmt = connection
            .prepare("SELECT path FROM projects WHERE id = ?1")
            .map_err(database_error)?;

        let row = stmt
            .query_row([id], |r| r.get::<_, String>(0))
            .optional()
            .map_err(database_error)?;
        Ok(row)
    }

    fn create_project(&self, project: &Project) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute(
                "INSERT INTO projects (id, name, path, created_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![project.id, project.name, project.path, project.created_at],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn delete_project(&self, id: &str) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute("DELETE FROM projects WHERE id = ?1", [id])
            .map_err(database_error)?;
        Ok(())
    }

    fn get_user_meta(&self, skill_id: &str) -> DomainResult<Option<UserSkillMeta>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let mut stmt = connection
            .prepare("SELECT category_id, user_notes FROM skills_user_meta WHERE skill_id = ?1")
            .map_err(database_error)?;

        let row = stmt
            .query_row([skill_id], |r| {
                let category_id: Option<String> = r.get(0)?;
                let user_notes: Option<String> = r.get(1)?;
                Ok(UserSkillMeta {
                    category_id,
                    user_notes,
                })
            })
            .optional()
            .map_err(database_error)?;

        Ok(row)
    }

    fn save_user_meta(
        &self,
        skill_id: &str,
        category_id: Option<&str>,
        user_notes: Option<&str>,
    ) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        connection
            .execute(
                "INSERT INTO skills_user_meta (skill_id, category_id, user_notes, updated_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(skill_id) DO UPDATE SET
                   category_id = excluded.category_id,
                   user_notes = excluded.user_notes,
                   updated_at = excluded.updated_at",
                rusqlite::params![skill_id, category_id, user_notes, now],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn get_project_skills(&self, project_id: &str) -> DomainResult<Vec<String>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let mut stmt = connection
            .prepare("SELECT skill_id FROM project_skills WHERE project_id = ?1")
            .map_err(database_error)?;

        let iter = stmt
            .query_map([project_id], |r| r.get::<_, String>(0))
            .map_err(database_error)?;

        let mut list = Vec::new();
        for item in iter {
            list.push(item.map_err(database_error)?);
        }
        Ok(list)
    }

    fn save_project_skill(
        &self,
        project_id: &str,
        skill_id: &str,
        enabled: bool,
    ) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;

        if enabled {
            let now = chrono::Utc::now().to_rfc3339();
            // First verify the skill exists in skills_user_meta to prevent FK violation
            connection
                .execute(
                    "INSERT OR IGNORE INTO skills_user_meta (skill_id, updated_at) VALUES (?1, ?2)",
                    [skill_id, &now],
                )
                .map_err(database_error)?;

            connection.execute(
                "INSERT OR IGNORE INTO project_skills (project_id, skill_id, enabled_at) VALUES (?1, ?2, ?3)",
                [project_id, skill_id, &now],
            ).map_err(database_error)?;
        } else {
            connection
                .execute(
                    "DELETE FROM project_skills WHERE project_id = ?1 AND skill_id = ?2",
                    [project_id, skill_id],
                )
                .map_err(database_error)?;
        }
        Ok(())
    }

    fn delete_user_meta(&self, skill_id: &str) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute(
                "DELETE FROM skills_user_meta WHERE skill_id = ?1",
                [skill_id],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn get_skill_package(&self, skill_id: &str) -> DomainResult<Option<SkillPackageRecord>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .query_row(
                "SELECT skill_id, source_kind, source_url, normalized_source, tracked_ref, installed_commit, trusted_commit, last_checked_at
                 FROM skill_packages WHERE skill_id = ?1",
                [skill_id],
                |row| {
                    let source_kind: String = row.get(1)?;
                    Ok(SkillPackageRecord {
                        skill_id: row.get(0)?,
                        source_kind: SourceKind::from_db(&source_kind),
                        source_url: row.get(2)?,
                        normalized_source: row.get(3)?,
                        tracked_ref: row.get(4)?,
                        installed_commit: row.get(5)?,
                        trusted_commit: row.get(6)?,
                        last_checked_at: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(database_error)
    }

    fn save_skill_package(&self, record: &SkillPackageRecord) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        connection
            .execute(
                "INSERT OR IGNORE INTO skills_user_meta (skill_id, updated_at) VALUES (?1, ?2)",
                rusqlite::params![record.skill_id, now],
            )
            .map_err(database_error)?;
        connection
            .execute(
                "INSERT INTO skill_packages
                   (skill_id, source_kind, source_url, normalized_source, tracked_ref, installed_commit, trusted_commit, last_checked_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(skill_id) DO UPDATE SET
                   source_kind = excluded.source_kind,
                   source_url = excluded.source_url,
                   normalized_source = excluded.normalized_source,
                   tracked_ref = excluded.tracked_ref,
                   installed_commit = excluded.installed_commit,
                   trusted_commit = excluded.trusted_commit,
                   last_checked_at = excluded.last_checked_at",
                rusqlite::params![
                    record.skill_id,
                    record.source_kind.as_str(),
                    record.source_url,
                    record.normalized_source,
                    record.tracked_ref,
                    record.installed_commit,
                    record.trusted_commit,
                    record.last_checked_at,
                ],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn find_skill_by_source(&self, source_url: &str) -> DomainResult<Option<String>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .query_row(
                "SELECT skill_id FROM skill_packages WHERE normalized_source = ?1",
                [source_url],
                |row| row.get(0),
            )
            .optional()
            .map_err(database_error)
    }

    fn migrate_git_skill_id(&self, old_id: &str, new_id: &str) -> DomainResult<()> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;

        connection
            .execute("PRAGMA foreign_keys = OFF", [])
            .map_err(database_error)?;

        let result = (|| {
            let tx = connection.transaction()?;

            tx.execute(
                "UPDATE skills_user_meta SET skill_id = ?1 WHERE skill_id = ?2",
                [new_id, old_id],
            )?;

            tx.execute(
                "UPDATE skill_packages SET skill_id = ?1 WHERE skill_id = ?2",
                [new_id, old_id],
            )?;

            tx.execute(
                "UPDATE project_skills SET skill_id = ?1 WHERE skill_id = ?2",
                [new_id, old_id],
            )?;

            tx.execute(
                "UPDATE project_skill_states SET skill_id = ?1 WHERE skill_id = ?2",
                [new_id, old_id],
            )?;

            tx.execute(
                "UPDATE skill_descriptions SET target_id = ?1 WHERE target_id = ?2 AND target_kind = 'package'",
                [new_id, old_id],
            )?;

            let old_member_prefix = format!("{}::%", old_id);
            tx.execute(
                "UPDATE skill_descriptions SET target_id = ?1 || SUBSTR(target_id, LENGTH(?2) + 1) WHERE target_id LIKE ?3 AND target_kind = 'member'",
                [new_id, old_id, &old_member_prefix],
            )?;

            tx.commit()?;
            Ok::<(), rusqlite::Error>(())
        })();

        let pragma_result = connection.execute("PRAGMA foreign_keys = ON", []);

        result.map_err(database_error)?;
        pragma_result.map_err(database_error)?;
        Ok(())
    }

    fn get_projects_using_skill(&self, skill_id: &str) -> DomainResult<Vec<String>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let mut statement = connection
            .prepare(
                "SELECT project_id FROM project_skills WHERE skill_id = ?1 ORDER BY project_id",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([skill_id], |row| row.get(0))
            .map_err(database_error)?;
        rows.collect::<Result<Vec<String>, _>>()
            .map_err(database_error)
    }

    fn save_project_skill_state(
        &self,
        project_id: &str,
        skill_id: &str,
        installed_commit: Option<&str>,
        sync_state: &str,
    ) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute(
                "INSERT INTO project_skill_states (project_id, skill_id, installed_commit, sync_state)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(project_id, skill_id) DO UPDATE SET
                   installed_commit = excluded.installed_commit,
                   sync_state = excluded.sync_state",
                rusqlite::params![project_id, skill_id, installed_commit, sync_state],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn get_categories(&self) -> DomainResult<Vec<Category>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let mut stmt = connection
            .prepare("SELECT id, name, created_at FROM categories ORDER BY created_at ASC")
            .map_err(database_error)?;
        let iter = stmt
            .query_map([], |r| {
                Ok(Category {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    created_at: r.get(2)?,
                })
            })
            .map_err(database_error)?;

        let mut list = Vec::new();
        for c in iter {
            list.push(c.map_err(database_error)?);
        }
        Ok(list)
    }

    fn create_category(&self, id: &str, name: &str, created_at: &str) -> DomainResult<Category> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute(
                "INSERT INTO categories (id, name, created_at) VALUES (?1, ?2, ?3)",
                [id, name, created_at],
            )
            .map_err(database_error)?;
        Ok(Category {
            id: id.to_string(),
            name: name.to_string(),
            created_at: created_at.to_string(),
        })
    }

    fn rename_category(&self, id: &str, name: &str) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute("UPDATE categories SET name = ?1 WHERE id = ?2", [name, id])
            .map_err(database_error)?;
        Ok(())
    }

    fn delete_category(&self, id: &str) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute("DELETE FROM categories WHERE id = ?1", [id])
            .map_err(database_error)?;
        Ok(())
    }

    fn get_custom_description(&self, target_id: &str) -> DomainResult<Option<String>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let row = connection
            .query_row(
                "SELECT custom_description FROM skill_descriptions WHERE target_id = ?1",
                [target_id],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(database_error)?;
        Ok(row)
    }

    fn save_custom_description(
        &self,
        target_id: &str,
        target_kind: &str,
        custom_description: &str,
    ) -> DomainResult<()> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        let tx = connection.transaction().map_err(database_error)?;
        tx.execute(
            "INSERT INTO skill_descriptions (target_id, target_kind, custom_description, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(target_id) DO UPDATE SET
               custom_description = excluded.custom_description,
               updated_at = excluded.updated_at",
            rusqlite::params![target_id, target_kind, custom_description, now],
        )
        .map_err(database_error)?;
        tx.commit().map_err(database_error)?;
        Ok(())
    }

    fn delete_custom_description(&self, target_id: &str) -> DomainResult<()> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let tx = connection.transaction().map_err(database_error)?;
        tx.execute(
            "DELETE FROM skill_descriptions WHERE target_id = ?1",
            [target_id],
        )
        .map_err(database_error)?;
        tx.commit().map_err(database_error)?;
        Ok(())
    }

    fn get_all_custom_descriptions(
        &self,
    ) -> DomainResult<Vec<crate::domain::skill::SkillDescriptionRecord>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let mut stmt = connection
            .prepare("SELECT target_id, target_kind, custom_description, updated_at FROM skill_descriptions ORDER BY target_id ASC")
            .map_err(database_error)?;
        let iter = stmt
            .query_map([], |row| {
                Ok(crate::domain::skill::SkillDescriptionRecord {
                    target_id: row.get(0)?,
                    target_kind: row.get(1)?,
                    custom_description: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })
            .map_err(database_error)?;
        let mut list = Vec::new();
        for item in iter {
            list.push(item.map_err(database_error)?);
        }
        Ok(list)
    }

    fn import_custom_descriptions(
        &self,
        records: Vec<crate::domain::skill::SkillDescriptionRecord>,
        conflict_strategy: &str,
    ) -> DomainResult<()> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let tx = connection.transaction().map_err(database_error)?;
        for record in records {
            let local_updated_at: Option<String> = tx
                .query_row(
                    "SELECT updated_at FROM skill_descriptions WHERE target_id = ?1",
                    [&record.target_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(database_error)?;

            let write_record = match local_updated_at {
                None => true,
                Some(local_time) => match conflict_strategy {
                    "keep_local" => false,
                    "keep_import" => true,
                    _ => {
                        // "keep_newer" (default)
                        if let (Ok(import_dt), Ok(local_dt)) = (
                            chrono::DateTime::parse_from_rfc3339(&record.updated_at),
                            chrono::DateTime::parse_from_rfc3339(&local_time),
                        ) {
                            import_dt > local_dt
                        } else {
                            record.updated_at > local_time
                        }
                    }
                },
            };

            if write_record {
                tx.execute(
                    "INSERT INTO skill_descriptions (target_id, target_kind, custom_description, updated_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(target_id) DO UPDATE SET
                       custom_description = excluded.custom_description,
                       updated_at = excluded.updated_at",
                    rusqlite::params![
                        record.target_id,
                        record.target_kind,
                        record.custom_description,
                        record.updated_at
                    ],
                )
                .map_err(database_error)?;
            }
        }
        tx.commit().map_err(database_error)?;
        Ok(())
    }

    fn delete_descriptions(&self, target_ids: &[String]) -> DomainResult<()> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let tx = connection.transaction().map_err(database_error)?;
        for id in target_ids {
            tx.execute("DELETE FROM skill_descriptions WHERE target_id = ?1", [id])
                .map_err(database_error)?;
        }
        tx.commit().map_err(database_error)?;
        Ok(())
    }
}

impl HarnessRepository for SqliteDatabase {
    fn get_harnesses(&self) -> DomainResult<Vec<crate::domain::harness::HarnessTemplateSummary>> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        let mut stmt = connection
            .prepare("SELECT id, name, description, work_type, created_from_preset, source_type, source_path, created_at, updated_at FROM harness_templates ORDER BY created_at ASC")
            .map_err(database_error)?;

        let iter = stmt
            .query_map([], |r| {
                Ok(crate::domain::harness::HarnessTemplateSummary {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    description: r.get(2)?,
                    work_type: r.get(3)?,
                    created_from_preset: r.get(4)?,
                    selected_modules: Vec::new(),
                    source_type: r.get(5)?,
                    source_path: r.get(6)?,
                    created_at: r.get(7)?,
                    updated_at: r.get(8)?,
                    file_count: 0,
                    has_agents_md: false,
                    has_manifest: false,
                    is_valid: false,
                })
            })
            .map_err(database_error)?;

        let mut list = Vec::new();
        for item in iter {
            list.push(item.map_err(database_error)?);
        }
        Ok(list)
    }

    fn save_harness(
        &self,
        summary: &crate::domain::harness::HarnessTemplateSummary,
    ) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute(
                "INSERT INTO harness_templates (id, name, description, work_type, created_from_preset, source_type, source_path, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   description = excluded.description,
                   work_type = excluded.work_type,
                   created_from_preset = excluded.created_from_preset,
                   source_type = excluded.source_type,
                   source_path = excluded.source_path,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    summary.id,
                    summary.name,
                    summary.description,
                    summary.work_type,
                    summary.created_from_preset,
                    summary.source_type,
                    summary.source_path,
                    summary.created_at,
                    summary.updated_at
                ],
            )
            .map_err(database_error)?;
        Ok(())
    }

    fn delete_harness(&self, id: &str) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute("DELETE FROM harness_templates WHERE id = ?1", [id])
            .map_err(database_error)?;
        Ok(())
    }
}

fn database_error(error: rusqlite::Error) -> DomainError {
    DomainError::Database(error.to_string())
}

#[cfg(test)]
mod tests {
    use crate::domain::health::{DatabasePort, DatabaseStatus};
    use crate::domain::ports::{HarnessRepository, SkillRepository};
    use crate::domain::skill::{SkillPackageRecord, SourceKind};

    use super::SqliteDatabase;

    #[test]
    fn in_memory_database_applies_schema_and_reports_ready() {
        let database = SqliteDatabase::open_in_memory().expect("database should initialize");

        assert_eq!(database.status().unwrap(), DatabaseStatus::Ready);
        assert!(database.has_table("_migrations").unwrap());
        assert!(database.has_table("projects").unwrap());
        assert!(database.has_table("task_runs").unwrap());
        assert!(database.has_table("categories").unwrap());
        assert!(database.has_table("skills_user_meta").unwrap());
        assert!(database.has_table("project_skills").unwrap());
        assert!(database.has_table("skill_packages").unwrap());
        assert!(database.has_table("project_skill_states").unwrap());
        assert!(database.has_table("skill_descriptions").unwrap());
        assert!(database.has_table("harness_templates").unwrap());
    }

    #[test]
    fn saves_and_loads_harness_templates() {
        let database = SqliteDatabase::open_in_memory().unwrap();
        let template = crate::domain::harness::HarnessTemplateSummary {
            id: "test-harness".into(),
            name: "Test Harness".into(),
            description: "Test Description".into(),
            work_type: "code".into(),
            created_from_preset: Some("code-feature-development".into()),
            selected_modules: Vec::new(),
            source_type: "local".into(),
            source_path: None,
            created_at: "2026-07-09T00:00:00Z".into(),
            updated_at: "2026-07-09T00:00:00Z".into(),
            file_count: 0,
            has_agents_md: false,
            has_manifest: false,
            is_valid: false,
        };

        database.save_harness(&template).unwrap();
        let list = database.get_harnesses().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "test-harness");
        assert_eq!(list[0].name, "Test Harness");
        assert_eq!(list[0].description, "Test Description");
        assert_eq!(list[0].work_type, "code");
        assert_eq!(
            list[0].created_from_preset.as_deref(),
            Some("code-feature-development")
        );
        assert_eq!(list[0].source_type, "local");

        // Delete
        database.delete_harness("test-harness").unwrap();
        let list2 = database.get_harnesses().unwrap();
        assert_eq!(list2.len(), 0);
    }

    #[test]
    fn saves_and_loads_skill_package_provenance() {
        let database = SqliteDatabase::open_in_memory().unwrap();
        let record = SkillPackageRecord {
            skill_id: "superpowers".into(),
            source_kind: SourceKind::Git,
            source_url: Some("github.com/obra/superpowers".into()),
            normalized_source: Some("github.com/obra/superpowers".into()),
            tracked_ref: Some("refs/tags/v6.1.1".into()),
            installed_commit: Some("abc123".into()),
            trusted_commit: None,
            last_checked_at: None,
        };

        database.save_skill_package(&record).unwrap();

        assert_eq!(
            database.get_skill_package("superpowers").unwrap(),
            Some(record)
        );
        assert_eq!(
            database
                .find_skill_by_source("github.com/obra/superpowers")
                .unwrap(),
            Some("superpowers".into())
        );
    }

    #[test]
    fn manages_custom_descriptions_lifecycle() {
        let database = SqliteDatabase::open_in_memory().unwrap();

        // Initially empty
        assert_eq!(database.get_custom_description("t-1").unwrap(), None);

        // Save new
        database
            .save_custom_description("t-1", "package", "My Test Package")
            .unwrap();
        assert_eq!(
            database.get_custom_description("t-1").unwrap(),
            Some("My Test Package".into())
        );

        // Update existing
        database
            .save_custom_description("t-1", "package", "My Updated Package")
            .unwrap();
        assert_eq!(
            database.get_custom_description("t-1").unwrap(),
            Some("My Updated Package".into())
        );

        // Delete
        database.delete_custom_description("t-1").unwrap();
        assert_eq!(database.get_custom_description("t-1").unwrap(), None);
    }

    #[test]
    fn migrates_git_skill_id_and_cascades_safely() {
        let database = SqliteDatabase::open_in_memory().unwrap();

        // Add category
        let conn = database.connection.lock().unwrap();
        conn.execute("INSERT INTO categories (id, name, created_at) VALUES ('cat1', 'Category 1', '2026-07-09T00:00:00Z')", []).unwrap();

        // Add project
        conn.execute("INSERT INTO projects (id, name, path, created_at) VALUES ('p1', 'Project 1', '/p1', '2026-07-09T00:00:00Z')", []).unwrap();

        // Add user meta (category dependency must exist)
        conn.execute("INSERT INTO skills_user_meta (skill_id, category_id, updated_at) VALUES ('old-id', 'cat1', '2026-07-09T00:00:00Z')", []).unwrap();

        // Add skill packages (user meta dependency must exist)
        conn.execute(
            "INSERT INTO skill_packages (skill_id, source_kind, source_url, normalized_source, tracked_ref, installed_commit)
             VALUES ('old-id', 'git', 'https://github.com/o/r.git', 'github.com/o/r', 'main', 'c1')",
            []
        ).unwrap();

        // Add project skills (project and user meta dependencies must exist)
        conn.execute("INSERT INTO project_skills (project_id, skill_id, enabled_at) VALUES ('p1', 'old-id', '2026-07-09T00:00:00Z')", []).unwrap();

        // Add project skill states (project skill dependency must exist)
        conn.execute("INSERT INTO project_skill_states (project_id, skill_id, sync_state) VALUES ('p1', 'old-id', 'current')", []).unwrap();

        conn.execute("INSERT INTO skill_descriptions (target_id, target_kind, custom_description, updated_at) VALUES ('old-id', 'package', 'Desc 1', '2026-07-09T00:00:00Z')", []).unwrap();
        conn.execute("INSERT INTO skill_descriptions (target_id, target_kind, custom_description, updated_at) VALUES ('old-id::sub1', 'member', 'Desc 2', '2026-07-09T00:00:00Z')", []).unwrap();

        drop(conn);

        // Perform migration
        database.migrate_git_skill_id("old-id", "new-id").unwrap();

        // Query results
        let conn = database.connection.lock().unwrap();

        let package_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill_packages WHERE skill_id = 'new-id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(package_count, 1);
        let old_package_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skill_packages WHERE skill_id = 'old-id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(old_package_count, 0);

        let meta_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skills_user_meta WHERE skill_id = 'new-id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(meta_count, 1);

        let proj_skill_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM project_skills WHERE skill_id = 'new-id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(proj_skill_count, 1);

        let state_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM project_skill_states WHERE skill_id = 'new-id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(state_count, 1);

        let package_desc: String = conn.query_row("SELECT custom_description FROM skill_descriptions WHERE target_id = 'new-id' AND target_kind = 'package'", [], |r| r.get(0)).unwrap();
        assert_eq!(package_desc, "Desc 1");

        let member_desc: String = conn.query_row("SELECT custom_description FROM skill_descriptions WHERE target_id = 'new-id::sub1' AND target_kind = 'member'", [], |r| r.get(0)).unwrap();
        assert_eq!(member_desc, "Desc 2");
    }
}
