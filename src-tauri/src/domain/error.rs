#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("database unavailable: {0}")]
    Database(String),
    #[error("application data directory is unavailable")]
    AppDataDirectory,
    #[error("operation unavailable: {0}")]
    Operation(String),
    #[error("git operation failed: {0}")]
    Git(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
