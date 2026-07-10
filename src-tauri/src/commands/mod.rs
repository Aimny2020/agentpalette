pub mod harnesses;
pub mod health;
pub mod projects;
pub mod skills;

use crate::domain::error::DomainError;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl From<DomainError> for CommandError {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::Database(details) => Self {
                code: "database_unavailable".into(),
                message: "本地数据库暂时不可用，请重试。".into(),
                details: Some(details),
            },
            DomainError::AppDataDirectory => Self {
                code: "app_data_unavailable".into(),
                message: "无法访问 AgentForge 数据目录。".into(),
                details: None,
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
        assert_eq!(error.message, "本地数据库暂时不可用，请重试。");
        assert_eq!(error.details.as_deref(), Some("file is locked"));
    }
}
