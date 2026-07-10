use crate::commands::health::AppState;
use crate::commands::CommandError;
use crate::domain::harness::{
    CreateHarnessTemplateInput, HarnessExtractOptions, HarnessFile, HarnessImportInspection,
    HarnessImportOptions, HarnessTemplateDetail, HarnessTemplateSummary, HarnessValidationReport,
};
use tauri::State;

#[tauri::command]
pub async fn get_harness_templates(
    state: State<'_, AppState>,
) -> Result<Vec<HarnessTemplateSummary>, CommandError> {
    state
        .harnesses
        .get_harness_templates()
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn inspect_harness_import(
    state: State<'_, AppState>,
    source_path: String,
) -> Result<HarnessImportInspection, CommandError> {
    state
        .harnesses
        .inspect_harness_import(&source_path)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn import_harness_from_folder(
    state: State<'_, AppState>,
    source_path: String,
    options: HarnessImportOptions,
) -> Result<HarnessTemplateDetail, CommandError> {
    state
        .harnesses
        .import_harness_from_folder(&source_path, options)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn extract_harness_from_project(
    state: State<'_, AppState>,
    project_id: String,
    options: HarnessExtractOptions,
) -> Result<HarnessTemplateDetail, CommandError> {
    state
        .harnesses
        .extract_harness_from_project(&project_id, options)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn create_harness_template(
    state: State<'_, AppState>,
    input: CreateHarnessTemplateInput,
) -> Result<HarnessTemplateDetail, CommandError> {
    state
        .harnesses
        .create_harness_template(input)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_harness_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<HarnessTemplateDetail, CommandError> {
    state
        .harnesses
        .get_harness_template(&template_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn read_harness_file(
    state: State<'_, AppState>,
    template_id: String,
    path: String,
) -> Result<HarnessFile, CommandError> {
    state
        .harnesses
        .read_harness_file(&template_id, &path)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn write_harness_file(
    state: State<'_, AppState>,
    template_id: String,
    path: String,
    content: String,
) -> Result<HarnessFile, CommandError> {
    state
        .harnesses
        .write_harness_file(&template_id, &path, &content)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn create_harness_file(
    state: State<'_, AppState>,
    template_id: String,
    path: String,
    kind: String,
) -> Result<HarnessFile, CommandError> {
    state
        .harnesses
        .create_harness_file(&template_id, &path, &kind)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn delete_harness_file(
    state: State<'_, AppState>,
    template_id: String,
    path: String,
) -> Result<(), CommandError> {
    state
        .harnesses
        .delete_harness_file(&template_id, &path)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn delete_harness_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<(), CommandError> {
    state
        .harnesses
        .delete_harness_template(&template_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn validate_harness_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<HarnessValidationReport, CommandError> {
    state
        .harnesses
        .validate_harness_template(&template_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn duplicate_harness_template(
    state: State<'_, AppState>,
    template_id: String,
    target_id: String,
    target_name: String,
) -> Result<HarnessTemplateDetail, CommandError> {
    state
        .harnesses
        .duplicate_harness_template(&template_id, &target_id, &target_name)
        .map_err(CommandError::from)
}
