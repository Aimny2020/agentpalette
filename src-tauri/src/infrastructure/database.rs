use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use rusqlite::OptionalExtension;

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::health::{DatabasePort, DatabaseStatus};
use crate::domain::ports::SkillRepository;
use crate::domain::skill::{Category, UserSkillMeta};
use crate::domain::project::Project;

const INITIAL_MIGRATION: &str = include_str!("../../migrations/001_initial.sql");
const SKILLS_MIGRATION: &str = include_str!("../../migrations/002_skills.sql");

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

    fn save_user_meta(&self, skill_id: &str, category_id: Option<&str>, user_notes: Option<&str>) -> DomainResult<()> {
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

    fn save_project_skill(&self, project_id: &str, skill_id: &str, enabled: bool) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        
        if enabled {
            let now = chrono::Utc::now().to_rfc3339();
            // First verify the skill exists in skills_user_meta to prevent FK violation
            connection.execute(
                "INSERT OR IGNORE INTO skills_user_meta (skill_id, updated_at) VALUES (?1, ?2)",
                [skill_id, &now],
            ).map_err(database_error)?;

            connection.execute(
                "INSERT OR IGNORE INTO project_skills (project_id, skill_id, enabled_at) VALUES (?1, ?2, ?3)",
                [project_id, skill_id, &now],
            ).map_err(database_error)?;
        } else {
            connection.execute(
                "DELETE FROM project_skills WHERE project_id = ?1 AND skill_id = ?2",
                [project_id, skill_id],
            ).map_err(database_error)?;
        }
        Ok(())
    }

    fn delete_user_meta(&self, skill_id: &str) -> DomainResult<()> {
        let connection = self
            .connection
            .lock()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        connection
            .execute("DELETE FROM skills_user_meta WHERE skill_id = ?1", [skill_id])
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
            .execute(
                "UPDATE categories SET name = ?1 WHERE id = ?2",
                [name, id],
            )
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
}

fn database_error(error: rusqlite::Error) -> DomainError {
    DomainError::Database(error.to_string())
}

#[cfg(test)]
mod tests {
    use crate::domain::health::{DatabasePort, DatabaseStatus};

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
    }
}
