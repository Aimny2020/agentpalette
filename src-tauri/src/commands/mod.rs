pub mod agents;
pub mod harnesses;
pub mod health;
pub mod projects;
pub mod settings;
pub mod skills;

use crate::domain::error::DomainError;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub details: Option<String>,
}

impl From<DomainError> for CommandError {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::Database(details) => Self {
                code: "database_unavailable".into(),
                details: Some(details),
            },
            DomainError::AppDataDirectory => Self {
                code: "app_data_unavailable".into(),
                details: None,
            },
            DomainError::Operation(details) => Self {
                code: "operation_unavailable".into(),
                details: Some(details),
            },
            DomainError::Git(details) => Self {
                code: "git_unavailable".into(),
                details: Some(details),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::error::DomainError;

    use super::CommandError;

    #[test]
    fn command_error_hides_internal_database_details() {
        let error = CommandError::from(DomainError::Database("file is locked".into()));

        assert_eq!(error.code, "database_unavailable");
        assert_eq!(error.code, "database_unavailable");
        assert_eq!(error.details.as_deref(), Some("file is locked"));
    }

    #[test]
    fn command_error_maps_git_failures() {
        let error = CommandError::from(DomainError::Git("connection reset".into()));

        assert_eq!(error.code, "git_unavailable");
        assert_eq!(error.details.as_deref(), Some("connection reset"));
    }
}
