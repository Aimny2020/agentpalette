use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use rusqlite::OptionalExtension;

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::health::{DatabasePort, DatabaseStatus};
use crate::domain::ports::SkillRepository;
use crate::domain::project::Project;
use crate::domain::skill::{Category, SkillPackageRecord, SourceKind, UserSkillMeta};

const INITIAL_MIGRATION: &str = include_str!("../../migrations/001_initial.sql");
const SKILLS_MIGRATION: &str = include_str!("../../migrations/002_skills.sql");
const SKILL_PACKAGES_MIGRATION: &str = include_str!("../../migrations/003_skill_packages.sql");
const SKILL_DESCRIPTIONS_MIGRATION: &str =
    include_str!("../../migrations/004_skill_descriptions.sql");

pub struct SqliteDatabase {
    connection: Mutex<Connection>,
}

impl SqliteDatabase {
    pub fn open(path: &Path) -> DomainResult<Self> {
        let connection = Connection::open(path).map_err(database_error)?;
        Self::initialize(connection)
    }

    #[cfg(test)]
    fn open_in_memory() -> DomainResult<Self> {
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

fn database_error(error: rusqlite::Error) -> DomainError {
    DomainError::Database(error.to_string())
}

#[cfg(test)]
mod tests {
    use crate::domain::health::{DatabasePort, DatabaseStatus};
    use crate::domain::ports::SkillRepository;
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
}
