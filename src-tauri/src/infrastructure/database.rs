use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
#[cfg(test)]
use rusqlite::OptionalExtension;

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::health::{DatabasePort, DatabaseStatus};

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
    }
}
