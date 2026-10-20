use tauri::State;

use crate::commands::health::AppState;
use crate::commands::CommandError;
use crate::domain::error::DomainError;
use crate::domain::settings::{
    LaunchPreferences, NetworkPreferences, GIT_PROXY_MODE_DIRECT, GIT_PROXY_MODE_MANUAL,
    GIT_PROXY_MODE_SYSTEM,
};
use crate::infrastructure::git_cli::{
    validate_manual_proxy_url, GitConnectionReport, GitEnvironmentReport, GitOptions,
};

#[tauri::command]
pub async fn get_launch_preferences(
    state: State<'_, AppState>,
) -> Result<LaunchPreferences, CommandError> {
    state
        .settings
        .get_launch_preferences()
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn save_launch_preferences(
    state: State<'_, AppState>,
    preferences: LaunchPreferences,
) -> Result<LaunchPreferences, CommandError> {
    state
        .settings
        .save_launch_preferences(&preferences)
        .map_err(CommandError::from)?;
    Ok(preferences)
}

#[tauri::command]
pub async fn get_network_preferences(
    state: State<'_, AppState>,
) -> Result<NetworkPreferences, CommandError> {
    state
        .settings
        .get_network_preferences()
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn save_network_preferences(
    state: State<'_, AppState>,
    preferences: NetworkPreferences,
) -> Result<NetworkPreferences, CommandError> {
    let mut preferences = preferences;
    match preferences.git_proxy_mode.as_str() {
        GIT_PROXY_MODE_MANUAL => {
            preferences.git_proxy_url = validate_manual_proxy_url(&preferences.git_proxy_url)
                .map_err(|error| CommandError::from(DomainError::Git(error)))?;
        }
        GIT_PROXY_MODE_SYSTEM | GIT_PROXY_MODE_DIRECT => {
            preferences.git_proxy_url.clear();
        }
        other => {
            return Err(CommandError::from(DomainError::Git(format!(
                "不支持的 Git 代理模式：{other}"
            ))))
        }
    }
    state
        .settings
        .save_network_preferences(&preferences)
        .map_err(CommandError::from)?;
    Ok(preferences)
}

#[tauri::command]
pub async fn get_git_environment(
    state: State<'_, AppState>,
) -> Result<GitEnvironmentReport, CommandError> {
    let preferences = state
        .settings
        .get_network_preferences()
        .map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || GitOptions::inspect_environment(&preferences))
        .await
        .map_err(|error| CommandError::from(DomainError::Operation(error.to_string())))
}

#[tauri::command]
pub async fn test_git_connection(
    state: State<'_, AppState>,
) -> Result<GitConnectionReport, CommandError> {
    let preferences = state
        .settings
        .get_network_preferences()
        .map_err(CommandError::from)?;
    tauri::async_runtime::spawn_blocking(move || match GitOptions::resolve(&preferences) {
        Ok(options) => options.test_connection(),
        Err(error) => GitConnectionReport {
            success: false,
            elapsed_ms: 0,
            proxy: None,
            message: Some(error.to_string()),
        },
    })
    .await
    .map_err(|error| CommandError::from(DomainError::Operation(error.to_string())))
}
