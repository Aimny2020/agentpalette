use tauri::State;
use crate::commands::health::AppState;
use crate::commands::CommandError;
use crate::domain::project::Project;

#[tauri::command]
pub async fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, CommandError> {
    state.repo.get_projects().map_err(CommandError::from)
}

#[tauri::command]
pub async fn add_project(
    state: State<'_, AppState>,
    path: String,
) -> Result<Project, CommandError> {
    let p = std::path::Path::new(&path);
    let name = p
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("未命名项目")
        .to_string();

    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let project = Project {
        id,
        name,
        path,
        created_at,
    };

    state.repo.create_project(&project).map_err(CommandError::from)?;
    Ok(project)
}

#[tauri::command]
pub async fn select_directory() -> Result<Option<String>, CommandError> {
    let result = rfd::AsyncFileDialog::new()
        .set_title("选择项目根目录")
        .pick_folder()
        .await;

    Ok(result.map(|path| path.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), CommandError> {
    state.repo.delete_project(&id).map_err(CommandError::from)
}
