use crate::commands::health::AppState;
use crate::commands::CommandError;
use crate::domain::harness::{
    CodeWorkModule, CreateHarnessTemplateInput, HarnessExtractOptions, HarnessFile,
    HarnessImportInspection, HarnessImportOptions, HarnessPreset, HarnessPresetFile,
    HarnessTemplateDetail, HarnessTemplateSummary, HarnessValidationReport,
};
use crate::domain::project_harness::{
    ProjectHarnessApplicationPreview, ProjectHarnessApplyInput, ProjectHarnessFile,
    ProjectHarnessStatus,
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
pub async fn get_harness_presets(
    state: State<'_, AppState>,
) -> Result<Vec<HarnessPreset>, CommandError> {
    Ok(state.harnesses.get_harness_presets())
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
    target_name: String,
) -> Result<HarnessTemplateDetail, CommandError> {
    state
        .harnesses
        .duplicate_harness_template(&template_id, &target_name)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_code_work_modules(
    state: State<'_, AppState>,
) -> Result<Vec<CodeWorkModule>, CommandError> {
    Ok(state.harnesses.get_code_work_modules())
}

#[tauri::command]
pub async fn get_code_work_shared_files(
    state: State<'_, AppState>,
) -> Result<Vec<HarnessPresetFile>, CommandError> {
    Ok(state.harnesses.get_code_work_shared_files())
}

#[tauri::command]
pub fn get_project_harness_status(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectHarnessStatus, CommandError> {
    state
        .project_harnesses
        .get_status(&project_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn preview_project_harness_application(
    state: State<'_, AppState>,
    project_id: String,
    template_id: String,
) -> Result<ProjectHarnessApplicationPreview, CommandError> {
    state
        .project_harnesses
        .preview_application(&project_id, &template_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn apply_project_harness(
    state: State<'_, AppState>,
    input: ProjectHarnessApplyInput,
) -> Result<ProjectHarnessStatus, CommandError> {
    state
        .project_harnesses
        .apply(input)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn read_project_harness_file(
    state: State<'_, AppState>,
    project_id: String,
    path: String,
) -> Result<ProjectHarnessFile, CommandError> {
    state
        .project_harnesses
        .read_file(&project_id, &path)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn write_project_harness_file(
    state: State<'_, AppState>,
    project_id: String,
    path: String,
    content: String,
) -> Result<ProjectHarnessFile, CommandError> {
    state
        .project_harnesses
        .write_file(&project_id, &path, &content)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn unmanage_project_harness(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<(), CommandError> {
    state
        .project_harnesses
        .unmanage(&project_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn adopt_project_harness(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectHarnessStatus, CommandError> {
    state
        .project_harnesses
        .adopt(&project_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn create_project_harness_file(
    state: State<'_, AppState>,
    project_id: String,
    path: String,
) -> Result<ProjectHarnessFile, CommandError> {
    state
        .project_harnesses
        .create_file(&project_id, &path)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn delete_project_harness_file(
    state: State<'_, AppState>,
    project_id: String,
    path: String,
    explicit_confirmation: bool,
) -> Result<(), CommandError> {
    state
        .project_harnesses
        .delete_file(&project_id, &path, explicit_confirmation)
        .map_err(CommandError::from)
}
