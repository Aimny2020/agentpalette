use crate::commands::health::AppState;
use crate::commands::CommandError;
use crate::domain::error::DomainError;
use crate::domain::skill::{
    Category, ImportInspection, Skill, SkillDescriptionRecord, SkillUpdate,
};
use tauri::State;

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
        state
            .skills
            .import_git_url(&source)
            .map_err(CommandError::from)
    } else {
        state
            .skills
            .import_local_folder(&source)
            .map_err(CommandError::from)
    }
}

#[tauri::command]
pub async fn inspect_skill_import(
    state: State<'_, AppState>,
    source: String,
    import_type: String,
) -> Result<ImportInspection, CommandError> {
    state
        .skills
        .inspect_import(&source, &import_type)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn delete_skill(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<(), CommandError> {
    state
        .skills
        .delete_skill(&skill_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn check_skill_updates(
    state: State<'_, AppState>,
) -> Result<Vec<SkillUpdate>, CommandError> {
    state
        .skills
        .check_skill_updates()
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn update_skill(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<SkillUpdate, CommandError> {
    state
        .skills
        .update_skill(&skill_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn trust_skill(state: State<'_, AppState>, skill_id: String) -> Result<(), CommandError> {
    state
        .skills
        .trust_skill(&skill_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn delete_skill_everywhere(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<(), CommandError> {
    state
        .skills
        .delete_skill_everywhere(&skill_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn update_skill_meta(
    state: State<'_, AppState>,
    skill_id: String,
    category_id: Option<String>,
    user_notes: Option<String>,
) -> Result<(), CommandError> {
    state
        .repo
        .save_user_meta(&skill_id, category_id.as_deref(), user_notes.as_deref())
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_project_skills(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<String>, CommandError> {
    state
        .repo
        .get_project_skills(&project_id)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn toggle_project_skill(
    state: State<'_, AppState>,
    project_id: String,
    skill_id: String,
    enabled: bool,
) -> Result<(), CommandError> {
    state
        .skills
        .toggle_project_skill(&project_id, &skill_id, enabled)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<Category>, CommandError> {
    state.repo.get_categories().map_err(CommandError::from)
}

#[tauri::command]
pub async fn create_category(
    state: State<'_, AppState>,
    name: String,
) -> Result<Category, CommandError> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    state
        .repo
        .create_category(&id, &name, &created_at)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn rename_category(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<(), CommandError> {
    state
        .repo
        .rename_category(&id, &name)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn delete_category(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    state.repo.delete_category(&id).map_err(CommandError::from)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SkillDescriptionsExport {
    pub schema_version: u32,
    pub exported_at: String,
    pub descriptions: Vec<SkillDescriptionRecord>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InvalidRecordInfo {
    pub target_id: Option<String>,
    pub target_kind: Option<String>,
    pub description: Option<String>,
    pub reason: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DescriptionsImportPreview {
    pub file_path: String,
    pub total_count: usize,
    pub new_count: usize,
    pub overwrite_count: usize,
    pub skip_count: usize,
    pub unassociated_count: usize,
    pub invalid_records: Vec<InvalidRecordInfo>,
    pub valid_records: Vec<SkillDescriptionRecord>,
}

#[tauri::command]
pub async fn save_custom_description(
    state: State<'_, AppState>,
    target_id: String,
    target_kind: String,
    description: Option<String>,
) -> Result<(), CommandError> {
    let trimmed = description.unwrap_or_default().trim().to_string();
    if trimmed.is_empty() {
        state
            .repo
            .delete_custom_description(&target_id)
            .map_err(CommandError::from)?;
        return Ok(());
    }
    if trimmed.chars().count() > 2000 {
        return Err(CommandError::from(DomainError::Database(
            "技能说明不能超过 2000 个字".into(),
        )));
    }
    if target_kind != "package" && target_kind != "member" {
        return Err(CommandError::from(DomainError::Database(
            "非法的 target_kind".into(),
        )));
    }
    state
        .repo
        .save_custom_description(&target_id, &target_kind, &trimmed)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn export_custom_descriptions(
    state: State<'_, AppState>,
) -> Result<Option<String>, CommandError> {
    let descriptions = state
        .repo
        .get_all_custom_descriptions()
        .map_err(CommandError::from)?;
    let export_data = SkillDescriptionsExport {
        schema_version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        descriptions,
    };
    let json_str = serde_json::to_string_pretty(&export_data)
        .map_err(|e| CommandError::from(DomainError::Database(e.to_string())))?;

    let file_path = rfd::AsyncFileDialog::new()
        .set_title("导出技能说明")
        .add_filter("JSON", &["json"])
        .save_file()
        .await;

    if let Some(path) = file_path {
        let path_buf = path.path().to_path_buf();
        std::fs::write(&path_buf, json_str)
            .map_err(|e| CommandError::from(DomainError::Database(e.to_string())))?;
        Ok(Some(path_buf.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn preview_custom_descriptions_import(
    state: State<'_, AppState>,
) -> Result<Option<DescriptionsImportPreview>, CommandError> {
    let file_handle = rfd::AsyncFileDialog::new()
        .set_title("选择导入的技能说明文件")
        .add_filter("JSON", &["json"])
        .pick_file()
        .await;

    let file = match file_handle {
        Some(f) => f,
        None => return Ok(None),
    };

    let path_buf = file.path().to_path_buf();
    let file_path_str = path_buf.to_string_lossy().to_string();
    let bytes = std::fs::read(&path_buf)
        .map_err(|e| CommandError::from(DomainError::Database(e.to_string())))?;

    let import_data: SkillDescriptionsExport = match serde_json::from_slice(&bytes) {
        Ok(data) => data,
        Err(e) => {
            return Err(CommandError::from(DomainError::Database(format!(
                "解析 JSON 失败: {e}"
            ))))
        }
    };

    if import_data.schema_version != 1 {
        return Err(CommandError::from(DomainError::Database(format!(
            "不支持的 schema_version: {}，当前仅支持版本 1",
            import_data.schema_version
        ))));
    }

    // Collect installed ids to determine if unassociated
    let installed_skills = state.skills.get_skills().map_err(CommandError::from)?;
    let mut installed_ids = std::collections::HashSet::new();
    for s in installed_skills {
        installed_ids.insert(s.id.clone());
        for m in s.members {
            installed_ids.insert(m.id.clone());
        }
    }

    let mut preview = DescriptionsImportPreview {
        file_path: file_path_str,
        total_count: import_data.descriptions.len(),
        new_count: 0,
        overwrite_count: 0,
        skip_count: 0,
        unassociated_count: 0,
        invalid_records: Vec::new(),
        valid_records: Vec::new(),
    };

    let mut seen_ids = std::collections::HashSet::new();

    for record in import_data.descriptions {
        let mut invalid = false;
        let mut reason = String::new();

        if record.target_id.trim().is_empty() {
            invalid = true;
            reason = "target_id 不能为空".into();
        } else if seen_ids.contains(&record.target_id) {
            invalid = true;
            reason = format!("重复的 target_id: {}", record.target_id);
        } else if record.target_kind != "package" && record.target_kind != "member" {
            invalid = true;
            reason = "target_kind 必须是 'package' 或 'member'".into();
        } else if record.custom_description.trim().is_empty() {
            invalid = true;
            reason = "说明内容不能为空".into();
        } else if record.custom_description.chars().count() > 2000 {
            invalid = true;
            reason = "说明内容超过 2000 个字符限制".into();
        } else if chrono::DateTime::parse_from_rfc3339(&record.updated_at).is_err() {
            invalid = true;
            reason = "更新时间 (updated_at) 格式非法，须为 RFC3339 格式".into();
        }

        if invalid {
            preview.invalid_records.push(InvalidRecordInfo {
                target_id: Some(record.target_id),
                target_kind: Some(record.target_kind),
                description: Some(record.custom_description),
                reason,
            });
            continue;
        }

        seen_ids.insert(record.target_id.clone());

        // Check association
        let is_associated = installed_ids.contains(&record.target_id);
        if !is_associated {
            preview.unassociated_count += 1;
        }

        // Check database conflict
        let local_updated_at = state
            .repo
            .get_custom_description(&record.target_id)
            .map_err(CommandError::from)?;
        match local_updated_at {
            None => {
                preview.new_count += 1;
            }
            Some(_) => {
                preview.overwrite_count += 1;
            }
        }
        preview.valid_records.push(record);
    }

    Ok(Some(preview))
}

#[tauri::command]
pub async fn confirm_custom_descriptions_import(
    state: State<'_, AppState>,
    records: Vec<SkillDescriptionRecord>,
    conflict_strategy: String,
) -> Result<(), CommandError> {
    state
        .repo
        .import_custom_descriptions(records, &conflict_strategy)
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn get_unassociated_descriptions_count(
    state: State<'_, AppState>,
) -> Result<usize, CommandError> {
    let descriptions = state
        .repo
        .get_all_custom_descriptions()
        .map_err(CommandError::from)?;
    let installed_skills = state.skills.get_skills().map_err(CommandError::from)?;
    let mut installed_ids = std::collections::HashSet::new();
    for s in installed_skills {
        installed_ids.insert(s.id.clone());
        for m in s.members {
            installed_ids.insert(m.id.clone());
        }
    }

    let count = descriptions
        .iter()
        .filter(|d| !installed_ids.contains(&d.target_id))
        .count();
    Ok(count)
}

#[tauri::command]
pub async fn clear_unassociated_descriptions(
    state: State<'_, AppState>,
) -> Result<usize, CommandError> {
    let descriptions = state
        .repo
        .get_all_custom_descriptions()
        .map_err(CommandError::from)?;
    let installed_skills = state.skills.get_skills().map_err(CommandError::from)?;
    let mut installed_ids = std::collections::HashSet::new();
    for s in installed_skills {
        installed_ids.insert(s.id.clone());
        for m in s.members {
            installed_ids.insert(m.id.clone());
        }
    }

    let unassociated_ids: Vec<String> = descriptions
        .into_iter()
        .filter(|d| !installed_ids.contains(&d.target_id))
        .map(|d| d.target_id)
        .collect();

    let len = unassociated_ids.len();
    state
        .repo
        .delete_descriptions(&unassociated_ids)
        .map_err(CommandError::from)?;
    Ok(len)
}
