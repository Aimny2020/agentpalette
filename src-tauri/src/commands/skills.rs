use tauri::State;
use crate::commands::health::AppState;
use crate::commands::CommandError;
use crate::domain::skill::{Skill, Category};

#[tauri::command]
pub async fn get_skills(state: State<'_, AppState>) -> Result<Vec<Skill>, CommandError> {
    state.skills.get_skills().map_err(CommandError::from)
}

#[tauri::command]
pub async fn import_skill(
    state: State<'_, AppState>,
    source: String,
    import_type: String,
) -> Result<String, CommandError> {
    if import_type == "git" {
        state.skills.import_git_url(&source).map_err(CommandError::from)
    } else {
        state.skills.import_local_folder(&source).map_err(CommandError::from)
    }
}

#[tauri::command]
pub async fn delete_skill(state: State<'_, AppState>, skill_id: String) -> Result<(), CommandError> {
    state.skills.delete_skill(&skill_id).map_err(CommandError::from)
}

#[tauri::command]
pub async fn update_skill_meta(
    state: State<'_, AppState>,
    skill_id: String,
    category_id: Option<String>,
    user_notes: Option<String>,
) -> Result<(), CommandError> {
    state.repo.save_user_meta(
        &skill_id,
        category_id.as_deref(),
        user_notes.as_deref(),
    ).map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_project_skills(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<String>, CommandError> {
    state.repo.get_project_skills(&project_id).map_err(CommandError::from)
}

#[tauri::command]
pub async fn toggle_project_skill(
    state: State<'_, AppState>,
    project_id: String,
    skill_id: String,
    enabled: bool,
) -> Result<(), CommandError> {
    state.repo.save_project_skill(&project_id, &skill_id, enabled).map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<Category>, CommandError> {
    state.repo.get_categories().map_err(CommandError::from)
}

#[tauri::command]
pub async fn create_category(state: State<'_, AppState>, name: String) -> Result<Category, CommandError> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    state.repo.create_category(&id, &name, &created_at).map_err(CommandError::from)
}

#[tauri::command]
pub async fn rename_category(state: State<'_, AppState>, id: String, name: String) -> Result<(), CommandError> {
    state.repo.rename_category(&id, &name).map_err(CommandError::from)
}

#[tauri::command]
pub async fn delete_category(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    state.repo.delete_category(&id).map_err(CommandError::from)
}
