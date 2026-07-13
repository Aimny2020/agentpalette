use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::harness::{
    CreateHarnessTemplateInput, HarnessExtractOptions, HarnessFile, HarnessFileSummary,
    HarnessImportInspection, HarnessImportOptions, HarnessManifest, HarnessTemplateDetail,
    HarnessTemplateFile, HarnessTemplateSummary, HarnessValidationReport,
};
use crate::domain::harness_presets::{built_in_harness_presets, find_harness_preset};
use crate::domain::ports::{HarnessRepository, SkillRepository};

pub struct HarnessService {
    repo: Arc<dyn HarnessRepository>,
    project_repo: Arc<dyn SkillRepository>,
    harnesses_dir: PathBuf,
}

impl HarnessService {
    pub fn new(repo: Arc<dyn HarnessRepository>, project_repo: Arc<dyn SkillRepository>) -> Self {
        let home = dirs::home_dir().expect("Failed to locate home directory");
        let harnesses_dir = home.join(".agent-forge").join("harnesses");
        if !harnesses_dir.exists() {
            fs::create_dir_all(&harnesses_dir).expect("Failed to create harnesses directory");
        }
        Self {
            repo,
            project_repo,
            harnesses_dir,
        }
    }

    #[cfg(test)]
    fn with_harnesses_dir(
        repo: Arc<dyn HarnessRepository>,
        project_repo: Arc<dyn SkillRepository>,
        harnesses_dir: PathBuf,
    ) -> Self {
        Self {
            repo,
            project_repo,
            harnesses_dir,
        }
    }

    fn safe_join(&self, template_id: &str, rel_path: &str) -> DomainResult<PathBuf> {
        let clean_path = std::path::Path::new(rel_path);
        if clean_path.is_absolute() || rel_path.contains("..") {
            return Err(DomainError::Database("Path traversal detected".into()));
        }
        let template_dir = self.harnesses_dir.join(template_id);
        Ok(template_dir.join(clean_path))
    }

    pub fn get_harness_templates(&self) -> DomainResult<Vec<HarnessTemplateSummary>> {
        if !self.harnesses_dir.exists() {
            return Ok(Vec::new());
        }

        let db_records = self.repo.get_harnesses()?;
        let mut db_map: std::collections::HashMap<String, HarnessTemplateSummary> =
            db_records.into_iter().map(|r| (r.id.clone(), r)).collect();

        let mut list = Vec::new();
        let entries =
            fs::read_dir(&self.harnesses_dir).map_err(|e| DomainError::Database(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let id = entry.file_name().to_string_lossy().into_owned();
            let has_agents_md = path.join("AGENTS.md").exists();
            let manifest_path = path.join("docs").join("harness.toml");
            let has_manifest = manifest_path.exists();

            let mut manifest: Option<HarnessManifest> = None;
            let mut manifest_parses = false;
            if has_manifest {
                if let Ok(toml_content) = fs::read_to_string(&manifest_path) {
                    if let Ok(parsed) = toml::from_str::<HarnessManifest>(&toml_content) {
                        manifest = Some(parsed);
                        manifest_parses = true;
                    }
                }
            }

            let file_count = count_files_recursive(&path).unwrap_or(0);
            let validation = self.validate_harness_template_internal(
                &id,
                &path,
                &manifest,
                has_agents_md,
                has_manifest,
                manifest_parses,
            )?;

            let (
                name,
                description,
                work_type,
                created_from_preset,
                selected_modules,
                source_type,
                source_path,
                created_at,
                updated_at,
            ) = if let Some(ref m) = manifest {
                let record = db_map
                    .remove(&id)
                    .unwrap_or_else(|| HarnessTemplateSummary {
                        id: id.clone(),
                        name: m.name.clone(),
                        description: m.description.clone(),
                        work_type: m.work_type.clone(),
                        created_from_preset: m.created_from_preset.clone(),
                        selected_modules: m.selected_modules.clone(),
                        source_type: m.source.clone(),
                        source_path: None,
                        created_at: chrono::Utc::now().to_rfc3339(),
                        updated_at: chrono::Utc::now().to_rfc3339(),
                        file_count,
                        has_agents_md,
                        has_manifest,
                        is_valid: validation.is_valid,
                    });
                (
                    m.name.clone(),
                    m.description.clone(),
                    m.work_type.clone(),
                    m.created_from_preset.clone().or(record.created_from_preset),
                    m.selected_modules.clone(),
                    record.source_type,
                    record.source_path,
                    record.created_at,
                    record.updated_at,
                )
            } else {
                let record = db_map
                    .remove(&id)
                    .unwrap_or_else(|| HarnessTemplateSummary {
                        id: id.clone(),
                        name: id.clone(),
                        description: "".into(),
                        work_type: "custom".into(),
                        created_from_preset: None,
                        selected_modules: Vec::new(),
                        source_type: "local".into(),
                        source_path: None,
                        created_at: chrono::Utc::now().to_rfc3339(),
                        updated_at: chrono::Utc::now().to_rfc3339(),
                        file_count,
                        has_agents_md,
                        has_manifest,
                        is_valid: validation.is_valid,
                    });
                (
                    record.name,
                    record.description,
                    record.work_type,
                    record.created_from_preset,
                    Vec::new(),
                    record.source_type,
                    record.source_path,
                    record.created_at,
                    record.updated_at,
                )
            };

            let summary = HarnessTemplateSummary {
                id,
                name,
                description,
                work_type,
                created_from_preset,
                selected_modules,
                source_type,
                source_path,
                created_at,
                updated_at,
                file_count,
                has_agents_md,
                has_manifest,
                is_valid: validation.is_valid,
            };

            self.repo.save_harness(&summary)?;
            list.push(summary);
        }

        // Clean up DB records that no longer exist on disk
        for (stale_id, _) in db_map {
            self.repo.delete_harness(&stale_id)?;
        }

        list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(list)
    }

    pub fn create_harness_template(
        &self,
        input: CreateHarnessTemplateInput,
    ) -> DomainResult<HarnessTemplateDetail> {
        let (selected_files, selected_modules) = self.resolve_creation_files(&input)?;
        let id = uuid::Uuid::new_v4().to_string();
        let template_dir = self.harnesses_dir.join(&id);

        fs::create_dir_all(&template_dir).map_err(|e| DomainError::Database(e.to_string()))?;
        fs::create_dir_all(template_dir.join("docs"))
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let agents_content = if input.work_type == "code" {
            self.generate_code_agents_content(&selected_modules, &selected_files)
        } else {
            self.generate_agents_content(&input.work_type, &selected_files)
        };
        fs::write(template_dir.join("AGENTS.md"), agents_content)
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut manifest = HarnessManifest {
            id: id.clone(),
            name: input.name.clone(),
            version: "1.0.0".into(),
            description: input.description.clone(),
            work_type: input.work_type.clone(),
            created_from_preset: if input.work_type == "code" {
                None
            } else {
                input.preset_id.clone()
            },
            selected_modules: if input.work_type == "code" {
                input.selected_modules.clone()
            } else {
                Vec::new()
            },
            source: "local".into(),
            required_files: vec!["AGENTS.md".into(), "docs/harness.toml".into()],
            files: Vec::new(),
        };

        for file in selected_files {
            let file_path = &file.path;
            manifest.files.push(HarnessTemplateFile {
                path: file_path.clone(),
                kind: file.kind.clone(),
                standard: true,
            });

            let target_path = template_dir.join(file_path);
            let parent = target_path.parent().unwrap();
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| DomainError::Database(e.to_string()))?;
            }
            fs::write(&target_path, file.content)
                .map_err(|e| DomainError::Database(e.to_string()))?;
        }

        let toml_str = toml::to_string(&manifest)
            .map_err(|e| DomainError::Database(format!("Failed to format manifest: {e}")))?;
        fs::write(template_dir.join("docs").join("harness.toml"), toml_str)
            .map_err(|e| DomainError::Database(e.to_string()))?;

        // Save index
        let now = chrono::Utc::now().to_rfc3339();
        let summary = HarnessTemplateSummary {
            id: id.clone(),
            name: input.name.clone(),
            description: input.description.clone(),
            work_type: input.work_type.clone(),
            created_from_preset: if input.work_type == "code" {
                None
            } else {
                input.preset_id.clone()
            },
            selected_modules: if input.work_type == "code" {
                input.selected_modules.clone()
            } else {
                Vec::new()
            },
            source_type: "local".into(),
            source_path: None,
            created_at: now.clone(),
            updated_at: now,
            file_count: count_files_recursive(&template_dir).unwrap_or(0),
            has_agents_md: true,
            has_manifest: true,
            is_valid: true,
        };
        self.repo.save_harness(&summary)?;

        self.get_harness_template(&id)
    }

    pub fn get_harness_presets(&self) -> Vec<crate::domain::harness::HarnessPreset> {
        built_in_harness_presets()
    }

    pub fn get_code_work_modules(&self) -> Vec<crate::domain::harness::CodeWorkModule> {
        crate::domain::harness_presets::built_in_code_work_modules()
    }

    pub fn get_code_work_shared_files(&self) -> Vec<crate::domain::harness::HarnessPresetFile> {
        crate::domain::harness_presets::code_work_shared_files()
    }

    fn resolve_creation_files(
        &self,
        input: &CreateHarnessTemplateInput,
    ) -> DomainResult<(
        Vec<crate::domain::harness::HarnessPresetFile>,
        Vec<crate::domain::harness::CodeWorkModule>,
    )> {
        match input.work_type.as_str() {
            "code" => {
                let modules =
                    self.resolve_code_modules(&input.selected_modules, input.preset_id.as_deref())?;
                let mut offered_files = crate::domain::harness_presets::code_work_shared_files();
                for module in &modules {
                    offered_files.extend(module.files.clone());
                }
                let mut seen_paths = std::collections::HashSet::new();
                let mut deduplicated_offered = Vec::new();
                for file in offered_files {
                    if seen_paths.insert(file.path.clone()) {
                        deduplicated_offered.push(file);
                    }
                }
                let selected_files =
                    self.select_preset_files(&input.optional_files, &deduplicated_offered)?;
                Ok((selected_files, modules))
            }
            "document" | "presentation" => {
                let preset = self.resolve_single_preset(
                    &input.work_type,
                    input.preset_id.as_deref(),
                    &input.selected_modules,
                )?;
                let selected_files =
                    self.select_preset_files(&input.optional_files, &preset.files)?;
                Ok((selected_files, Vec::new()))
            }
            "custom" => {
                self.resolve_custom_files(input.preset_id.as_deref(), &input.selected_modules)?;
                let available_files = built_in_harness_presets()
                    .into_iter()
                    .flat_map(|item| item.files)
                    .collect::<Vec<_>>();
                let selected_files =
                    self.select_preset_files(&input.optional_files, &available_files)?;
                Ok((selected_files, Vec::new()))
            }
            _ => Err(DomainError::Database(
                "Unsupported Harness work type".into(),
            )),
        }
    }

    fn resolve_code_modules(
        &self,
        selected_modules: &[String],
        preset_id: Option<&str>,
    ) -> DomainResult<Vec<crate::domain::harness::CodeWorkModule>> {
        if selected_modules.is_empty() {
            return Err(DomainError::Database(
                "Code Work requires at least one module".into(),
            ));
        }
        if preset_id.is_some() {
            return Err(DomainError::Database(
                "Code Work cannot use a built-in preset".into(),
            ));
        }

        let mut seen = std::collections::HashSet::new();
        let mut modules = Vec::new();
        for id in selected_modules {
            if !seen.insert(id.clone()) {
                return Err(DomainError::Database(format!(
                    "Duplicate module ID '{}'",
                    id
                )));
            }
            let module = crate::domain::harness_presets::find_code_work_module(id)
                .ok_or_else(|| DomainError::Database(format!("Unknown module ID '{}'", id)))?;
            modules.push(module);
        }

        Ok(modules)
    }

    fn resolve_single_preset(
        &self,
        work_type: &str,
        preset_id: Option<&str>,
        selected_modules: &[String],
    ) -> DomainResult<crate::domain::harness::HarnessPreset> {
        if !selected_modules.is_empty() {
            return Err(DomainError::Database(format!(
                "Cannot select modules for {} work",
                work_type
            )));
        }
        let preset_id = preset_id.ok_or_else(|| {
            DomainError::Database("A built-in preset is required for this work type".into())
        })?;
        let preset = find_harness_preset(preset_id).ok_or_else(|| {
            DomainError::Database(format!("Unknown Harness preset '{preset_id}'"))
        })?;
        if preset.work_type != work_type {
            return Err(DomainError::Database(format!(
                "Preset '{}' belongs to work type '{}', not '{}'",
                preset.id, preset.work_type, work_type
            )));
        }
        Ok(preset)
    }

    fn resolve_custom_files(
        &self,
        preset_id: Option<&str>,
        selected_modules: &[String],
    ) -> DomainResult<()> {
        if preset_id.is_some() {
            return Err(DomainError::Database(
                "Custom Work cannot use a built-in preset".into(),
            ));
        }
        if !selected_modules.is_empty() {
            return Err(DomainError::Database(
                "Custom Work cannot select modules".into(),
            ));
        }
        Ok(())
    }

    fn select_preset_files(
        &self,
        selected_paths: &[String],
        available_files: &[crate::domain::harness::HarnessPresetFile],
    ) -> DomainResult<Vec<crate::domain::harness::HarnessPresetFile>> {
        let mut selected = Vec::new();
        for path in selected_paths {
            let file = available_files
                .iter()
                .find(|candidate| candidate.path == *path)
                .ok_or_else(|| {
                    DomainError::Database(format!(
                        "File '{}' is not available in the selected Harness preset",
                        path
                    ))
                })?;
            if !selected
                .iter()
                .any(|item: &crate::domain::harness::HarnessPresetFile| item.path == file.path)
            {
                selected.push(file.clone());
            }
        }
        Ok(selected)
    }

    fn generate_agents_content(
        &self,
        work_type: &str,
        files: &[crate::domain::harness::HarnessPresetFile],
    ) -> String {
        let mut content = format!(
            "# Agent Workspace Instructions\n\nThis is a {work_type} Harness for long-running, evidence-based work.\n\n## Startup Workflow\n\n1. Read this file completely.\n2. Read the selected status, scope, and verification files listed below.\n3. Confirm the current verified state before starting work.\n4. Work on one active item at a time.\n\n## Work Rules\n\n- Keep changes within the active work item.\n- Do not claim completion without verification evidence.\n- Record decisions, blockers, and next steps in the selected state files.\n\n## Selected Harness Files\n\n"
        );
        for file in files {
            content.push_str(&format!(
                "- **{}**: [{}]({})\n",
                file.label, file.path, file.path
            ));
        }
        content.push_str(
            "\n## Definition of Done\n\nThe work is complete only when the selected verification and quality criteria pass and the evidence is recorded.\n\n## End Of Session\n\nUpdate the selected task-status and session-handoff files, record unresolved risks, and leave the next step explicit.\n",
        );
        content
    }

    fn generate_code_agents_content(
        &self,
        selected_modules: &[crate::domain::harness::CodeWorkModule],
        selected_files: &[crate::domain::harness::HarnessPresetFile],
    ) -> String {
        let mut content = String::new();

        content.push_str("# Agent Workspace Instructions\n\n");

        content.push_str("## Shared Core Rules\n\n");
        content.push_str("- Startup: Read this file completely.\n");
        content
            .push_str("- Read the selected status, scope, and verification files listed below.\n");
        content.push_str("- Scope Discipline: Work on one active item at a time.\n");
        content.push_str("- State Updates: Record decisions, blockers, and next steps in the selected state files.\n");
        content.push_str(
            "- Verification Evidence: Do not claim completion without verification evidence.\n",
        );
        content.push_str(
            "- Session Handoff: Always leave the next step explicit and update status.\n\n",
        );

        content.push_str("## Task Classification\n\n");
        content.push_str("Use these signals to determine which selected module rules apply:\n");
        for module in selected_modules {
            match module.id.as_str() {
                "technical-design" => {
                    content.push_str("- Architecture, boundaries, alternatives, or design decisions: Technical Design Role.\n");
                }
                "feature-development" => {
                    content.push_str("- Implementation, bug fixes, refactors, or tests: Feature Development Role.\n");
                }
                "code-review" => {
                    content.push_str("- Review, acceptance, quality inspection, or verification of a change: Code Review Role.\n");
                }
                _ => {}
            }
        }
        content.push('\n');

        for module in selected_modules {
            match module.id.as_str() {
                "technical-design" => {
                    content.push_str("## Technical Design Role\n\n");
                    content.push_str(&module.agent_instructions);
                    content.push_str("\n\n");
                }
                "feature-development" => {
                    content.push_str("## Feature Development Role\n\n");
                    content.push_str(&module.agent_instructions);
                    content.push_str("\n\n");
                }
                "code-review" => {
                    content.push_str("## Code Review Role\n\n");
                    content.push_str(&module.agent_instructions);
                    content.push_str("\n\n");
                }
                _ => {}
            }
        }

        content.push_str("## Multi-Module Order\n\n");
        if selected_modules.len() > 1 {
            content.push_str("Technical Design -> Feature Development -> Code Review\n");
        } else {
            content.push_str("Single module execution.\n");
        }
        if selected_modules.iter().any(|m| m.id == "code-review") {
            content
                .push_str("Failed review returns work to Feature Development before re-review.\n");
        }
        content.push('\n');

        content.push_str("## Selected Harness Files\n\n");
        for file in selected_files {
            content.push_str(&format!(
                "- **{}**: [{}]({})\n",
                file.label, file.path, file.path
            ));
        }
        content.push('\n');

        content.push_str("## Definition of Done\n\n");
        content.push_str("The work is complete only when the selected verification and quality criteria pass and the evidence is recorded.\n\n");

        content.push_str("## End Of Session\n\n");
        content.push_str("Update the selected task-status and session-handoff files, record unresolved risks, and leave the next step explicit.\n");

        content
    }

    pub fn get_harness_template(&self, template_id: &str) -> DomainResult<HarnessTemplateDetail> {
        let template_dir = self.harnesses_dir.join(template_id);
        if !template_dir.exists() {
            return Err(DomainError::Database(format!(
                "Harness template directory '{}' not found",
                template_id
            )));
        }

        // Get DB details
        let db_records = self.repo.get_harnesses()?;
        let summary = db_records
            .into_iter()
            .find(|r| r.id == template_id)
            .ok_or_else(|| {
                DomainError::Database(format!("Template '{}' index not found in DB", template_id))
            })?;

        // Read files recursively
        let mut file_summaries = Vec::new();
        let manifest_path = template_dir.join("docs").join("harness.toml");
        let has_manifest = manifest_path.exists();

        let mut manifest: Option<HarnessManifest> = None;
        let mut manifest_parses = false;
        if has_manifest {
            if let Ok(toml_content) = fs::read_to_string(&manifest_path) {
                if let Ok(parsed) = toml::from_str::<HarnessManifest>(&toml_content) {
                    manifest = Some(parsed);
                    manifest_parses = true;
                }
            }
        }

        self.list_files_recursive(&template_dir, &template_dir, &manifest, &mut file_summaries)?;

        let has_agents_md = template_dir.join("AGENTS.md").exists();
        let validation = self.validate_harness_template_internal(
            template_id,
            &template_dir,
            &manifest,
            has_agents_md,
            has_manifest,
            manifest_parses,
        )?;

        Ok(HarnessTemplateDetail {
            id: template_id.to_string(),
            name: summary.name,
            description: summary.description,
            work_type: summary.work_type,
            created_from_preset: summary.created_from_preset,
            selected_modules: manifest
                .as_ref()
                .map(|m| m.selected_modules.clone())
                .unwrap_or_default(),
            source_type: summary.source_type,
            source_path: summary.source_path,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
            files: file_summaries,
            validation,
        })
    }

    fn list_files_recursive(
        &self,
        base_dir: &Path,
        current_dir: &Path,
        manifest: &Option<HarnessManifest>,
        list: &mut Vec<HarnessFileSummary>,
    ) -> DomainResult<()> {
        let entries =
            fs::read_dir(current_dir).map_err(|e| DomainError::Database(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
            let path = entry.path();
            let name_os = entry.file_name();
            let name = name_os.to_string_lossy();
            if name == ".git" || name == ".DS_Store" {
                continue;
            }

            if path.is_dir() {
                self.list_files_recursive(base_dir, &path, manifest, list)?;
            } else {
                let rel_path = path
                    .strip_prefix(base_dir)
                    .map_err(|e| DomainError::Database(e.to_string()))?
                    .to_string_lossy()
                    .into_owned();

                let metadata =
                    fs::metadata(&path).map_err(|e| DomainError::Database(e.to_string()))?;

                let is_standard = if let Some(ref m) = manifest {
                    m.required_files.contains(&rel_path)
                        || m.files.iter().any(|f| f.path == rel_path)
                } else {
                    rel_path == "AGENTS.md" || rel_path == "docs/harness.toml"
                };

                list.push(HarnessFileSummary {
                    path: rel_path,
                    size: metadata.len(),
                    is_standard,
                });
            }
        }
        Ok(())
    }

    pub fn read_harness_file(&self, template_id: &str, path: &str) -> DomainResult<HarnessFile> {
        let target_path = self.safe_join(template_id, path)?;
        if !target_path.exists() {
            return Err(DomainError::Database(format!(
                "File '{}' does not exist in template '{}'",
                path, template_id
            )));
        }

        let content =
            fs::read_to_string(&target_path).map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(HarnessFile {
            path: path.to_string(),
            content,
        })
    }

    pub fn write_harness_file(
        &self,
        template_id: &str,
        path: &str,
        content: &str,
    ) -> DomainResult<HarnessFile> {
        let target_path = self.safe_join(template_id, path)?;
        let parent = target_path.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| DomainError::Database(e.to_string()))?;
        }

        fs::write(&target_path, content).map_err(|e| DomainError::Database(e.to_string()))?;

        // Update DB summary updated_at time
        let db_records = self.repo.get_harnesses()?;
        if let Some(mut summary) = db_records.into_iter().find(|r| r.id == template_id) {
            summary.updated_at = chrono::Utc::now().to_rfc3339();
            summary.file_count =
                count_files_recursive(&self.harnesses_dir.join(template_id)).unwrap_or(0);
            self.repo.save_harness(&summary)?;
        }

        Ok(HarnessFile {
            path: path.to_string(),
            content: content.to_string(),
        })
    }

    pub fn create_harness_file(
        &self,
        template_id: &str,
        path: &str,
        kind: &str,
    ) -> DomainResult<HarnessFile> {
        let target_path = self.safe_join(template_id, path)?;
        if target_path.exists() {
            return Err(DomainError::Database(format!(
                "File '{}' already exists in template '{}'",
                path, template_id
            )));
        }

        let parent = target_path.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| DomainError::Database(e.to_string()))?;
        }

        let default_content = match kind {
            "json" => "{\n  \n}",
            "toml" => "",
            _ => "",
        };

        fs::write(&target_path, default_content)
            .map_err(|e| DomainError::Database(e.to_string()))?;

        // Update harness.toml file entries if manifest is writable
        let manifest_path = self
            .harnesses_dir
            .join(template_id)
            .join("docs")
            .join("harness.toml");
        if manifest_path.exists() {
            if let Ok(toml_content) = fs::read_to_string(&manifest_path) {
                if let Ok(mut manifest) = toml::from_str::<HarnessManifest>(&toml_content) {
                    if !manifest.required_files.contains(&path.to_string())
                        && !manifest.files.iter().any(|f| f.path == path)
                    {
                        manifest.files.push(HarnessTemplateFile {
                            path: path.to_string(),
                            kind: kind.to_string(),
                            standard: false,
                        });
                        if let Ok(updated_toml) = toml::to_string(&manifest) {
                            let _ = fs::write(&manifest_path, updated_toml);
                        }
                    }
                }
            }
        }

        // Update DB summary updated_at time
        let db_records = self.repo.get_harnesses()?;
        if let Some(mut summary) = db_records.into_iter().find(|r| r.id == template_id) {
            summary.updated_at = chrono::Utc::now().to_rfc3339();
            summary.file_count =
                count_files_recursive(&self.harnesses_dir.join(template_id)).unwrap_or(0);
            self.repo.save_harness(&summary)?;
        }

        Ok(HarnessFile {
            path: path.to_string(),
            content: default_content.to_string(),
        })
    }

    pub fn delete_harness_file(&self, template_id: &str, path: &str) -> DomainResult<()> {
        let target_path = self.safe_join(template_id, path)?;
        if !target_path.exists() {
            return Err(DomainError::Database(format!(
                "File '{}' does not exist in template '{}'",
                path, template_id
            )));
        }

        if path == "AGENTS.md" || path == "docs/harness.toml" {
            return Err(DomainError::Database(
                "Cannot delete required harness template files".into(),
            ));
        }

        fs::remove_file(&target_path).map_err(|e| DomainError::Database(e.to_string()))?;

        // Update harness.toml file entries
        let manifest_path = self
            .harnesses_dir
            .join(template_id)
            .join("docs")
            .join("harness.toml");
        if manifest_path.exists() {
            if let Ok(toml_content) = fs::read_to_string(&manifest_path) {
                if let Ok(mut manifest) = toml::from_str::<HarnessManifest>(&toml_content) {
                    let original_len = manifest.files.len();
                    manifest.files.retain(|f| f.path != path);
                    if manifest.files.len() != original_len {
                        if let Ok(updated_toml) = toml::to_string(&manifest) {
                            let _ = fs::write(&manifest_path, updated_toml);
                        }
                    }
                }
            }
        }

        // Update DB summary updated_at time
        let db_records = self.repo.get_harnesses()?;
        if let Some(mut summary) = db_records.into_iter().find(|r| r.id == template_id) {
            summary.updated_at = chrono::Utc::now().to_rfc3339();
            summary.file_count =
                count_files_recursive(&self.harnesses_dir.join(template_id)).unwrap_or(0);
            self.repo.save_harness(&summary)?;
        }

        Ok(())
    }

    pub fn delete_harness_template(&self, template_id: &str) -> DomainResult<()> {
        let template_dir = self.harnesses_dir.join(template_id);
        if template_dir.exists() {
            fs::remove_dir_all(&template_dir).map_err(|e| DomainError::Database(e.to_string()))?;
        }

        self.repo.delete_harness(template_id)?;
        Ok(())
    }

    pub fn duplicate_harness_template(
        &self,
        template_id: &str,
        target_name: &str,
    ) -> DomainResult<HarnessTemplateDetail> {
        let src = self.harnesses_dir.join(template_id);
        let target_id = uuid::Uuid::new_v4().to_string();
        let dst = self.harnesses_dir.join(&target_id);
        if !src.exists() {
            return Err(DomainError::Database(format!(
                "Source template '{}' not found",
                template_id
            )));
        }
        if dst.exists() {
            return Err(DomainError::Database(format!(
                "Destination template '{}' already exists",
                target_id
            )));
        }
        self.copy_harness_dir(&src, &dst)?;

        // Update new docs/harness.toml manifest
        let mut manifest_modules = Vec::new();
        let manifest_path = dst.join("docs").join("harness.toml");
        if manifest_path.exists() {
            if let Ok(toml_content) = fs::read_to_string(&manifest_path) {
                if let Ok(mut manifest) = toml::from_str::<HarnessManifest>(&toml_content) {
                    manifest.id = target_id.clone();
                    manifest.name = target_name.to_string();
                    manifest_modules = manifest.selected_modules.clone();
                    if let Ok(updated_toml) = toml::to_string(&manifest) {
                        let _ = fs::write(&manifest_path, updated_toml);
                    }
                }
            }
        }

        let db_records = self.repo.get_harnesses()?;
        let now = chrono::Utc::now().to_rfc3339();
        let (description, work_type, created_from_preset, db_modules, source_type) =
            if let Some(old) = db_records.into_iter().find(|r| r.id == template_id) {
                (
                    old.description,
                    old.work_type,
                    old.created_from_preset,
                    old.selected_modules,
                    old.source_type,
                )
            } else {
                ("".into(), "custom".into(), None, Vec::new(), "local".into())
            };

        let selected_modules = if manifest_path.exists() {
            manifest_modules
        } else {
            db_modules
        };

        let summary = HarnessTemplateSummary {
            id: target_id.clone(),
            name: target_name.to_string(),
            description,
            work_type,
            created_from_preset,
            selected_modules,
            source_type,
            source_path: None,
            created_at: now.clone(),
            updated_at: now,
            file_count: count_files_recursive(&dst).unwrap_or(0),
            has_agents_md: dst.join("AGENTS.md").exists(),
            has_manifest: manifest_path.exists(),
            is_valid: true,
        };

        self.repo.save_harness(&summary)?;
        self.get_harness_template(&target_id)
    }

    pub fn validate_harness_template(
        &self,
        template_id: &str,
    ) -> DomainResult<HarnessValidationReport> {
        let template_dir = self.harnesses_dir.join(template_id);
        if !template_dir.exists() {
            return Err(DomainError::Database(format!(
                "Harness template directory '{}' not found",
                template_id
            )));
        }

        let has_agents_md = template_dir.join("AGENTS.md").exists();
        let manifest_path = template_dir.join("docs").join("harness.toml");
        let has_manifest = manifest_path.exists();

        let mut manifest: Option<HarnessManifest> = None;
        let mut manifest_parses = false;
        if has_manifest {
            if let Ok(toml_content) = fs::read_to_string(&manifest_path) {
                if let Ok(parsed) = toml::from_str::<HarnessManifest>(&toml_content) {
                    manifest = Some(parsed);
                    manifest_parses = true;
                }
            }
        }

        self.validate_harness_template_internal(
            template_id,
            &template_dir,
            &manifest,
            has_agents_md,
            has_manifest,
            manifest_parses,
        )
    }

    fn validate_harness_template_internal(
        &self,
        _template_id: &str,
        template_dir: &Path,
        manifest: &Option<HarnessManifest>,
        has_agents_md: bool,
        has_manifest: bool,
        manifest_parses: bool,
    ) -> DomainResult<HarnessValidationReport> {
        let mut missing_required_files = Vec::new();
        let mut syntax_errors = Vec::new();
        let mut warnings = Vec::new();

        if !has_agents_md {
            missing_required_files.push("AGENTS.md".into());
        }
        if !has_manifest {
            missing_required_files.push("docs/harness.toml".into());
        }

        let mut files_to_validate = Vec::new();

        if let Some(ref m) = manifest {
            for req in &m.required_files {
                let path = template_dir.join(req);
                if !path.exists() {
                    missing_required_files.push(req.clone());
                } else {
                    files_to_validate.push(req.clone());
                }
            }
            for file_entry in &m.files {
                let path = template_dir.join(&file_entry.path);
                if !path.exists() {
                    missing_required_files.push(file_entry.path.clone());
                } else {
                    files_to_validate.push(file_entry.path.clone());
                }
            }
        }

        // Validate syntax of JSON and TOML files
        for rel_path in files_to_validate {
            let full_path = template_dir.join(&rel_path);
            if !full_path.exists() {
                continue;
            }
            if rel_path.ends_with(".json") {
                if let Ok(bytes) = fs::read(&full_path) {
                    if let Err(e) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        syntax_errors.push(format!("{}: Invalid JSON syntax ({})", rel_path, e));
                    }
                }
            } else if rel_path.ends_with(".toml") {
                if let Ok(toml_content) = fs::read_to_string(&full_path) {
                    if let Err(e) = toml::from_str::<toml::Value>(&toml_content) {
                        syntax_errors.push(format!("{}: Invalid TOML syntax ({})", rel_path, e));
                    }
                }
            }
        }

        // Advisory check: check if AGENTS.md references optional standard files
        if has_agents_md && manifest.as_ref().map(|m| m.work_type.as_str()) != Some("code") {
            if let Ok(agents_content) = fs::read_to_string(template_dir.join("AGENTS.md")) {
                let optional_standard_files = vec![
                    "docs/architecture.md",
                    "docs/feature_list.json",
                    "docs/task-status.md",
                    "docs/verification.md",
                    "docs/risk-rules.md",
                    "docs/agent-profile.md",
                ];
                for file_path in optional_standard_files {
                    let check_path = template_dir.join(file_path);
                    if check_path.exists() {
                        // Check if AGENTS.md contains the filename
                        if !agents_content.contains(file_path) {
                            warnings.push(format!(
                                "AGENTS.md appears to be missing a reference to existing optional file '{}'",
                                file_path
                            ));
                        }
                    }
                }
            }
        }

        if let Some(ref m) = manifest {
            if m.work_type == "code" {
                if m.selected_modules.is_empty() {
                    warnings
                        .push("Legacy template: Code Work manifest has no selected modules".into());
                }

                let mut seen = std::collections::HashSet::new();
                for mod_id in &m.selected_modules {
                    if !seen.insert(mod_id.clone()) {
                        warnings.push(format!("Duplicate selected module ID '{}'", mod_id));
                    }
                }

                let agents_content = if has_agents_md {
                    fs::read_to_string(template_dir.join("AGENTS.md")).ok()
                } else {
                    None
                };

                let mut validated_modules = std::collections::HashSet::new();
                for mod_id in &m.selected_modules {
                    if !validated_modules.insert(mod_id.clone()) {
                        continue;
                    }
                    if let Some(module) =
                        crate::domain::harness_presets::find_code_work_module(mod_id)
                    {
                        for mod_file in &module.files {
                            let path = template_dir.join(&mod_file.path);
                            if !path.exists() {
                                warnings.push(format!(
                                    "Missing dedicated file '{}' for module '{}'",
                                    mod_file.path, mod_id
                                ));
                            }
                            if let Some(ref content) = agents_content {
                                if !content.contains(&mod_file.path) {
                                    warnings.push(format!(
                                        "AGENTS.md does not reference dedicated file '{}' for module '{}'",
                                        mod_file.path, mod_id
                                    ));
                                }
                            }
                        }

                        if let Some(ref content) = agents_content {
                            let role_heading = format!("## {} Role", module.name);
                            if !content.contains(&role_heading) {
                                warnings.push(format!(
                                    "AGENTS.md does not contain role heading '{}' for module '{}'",
                                    role_heading, mod_id
                                ));
                            }
                        }
                    } else {
                        warnings.push(format!("Unknown selected module ID '{}'", mod_id));
                    }
                }
            }
        }

        let is_valid = has_agents_md
            && has_manifest
            && manifest_parses
            && missing_required_files.is_empty()
            && syntax_errors.is_empty();

        Ok(HarnessValidationReport {
            has_agents_md,
            has_manifest,
            manifest_parses,
            missing_required_files,
            syntax_errors,
            warnings,
            is_valid,
        })
    }

    pub fn inspect_harness_import(
        &self,
        source_path: &str,
    ) -> DomainResult<HarnessImportInspection> {
        let src = Path::new(source_path);
        if !src.exists() || !src.is_dir() {
            return Err(DomainError::Database(
                "Source folder does not exist or is not a directory".into(),
            ));
        }

        let has_agents_md = src.join("AGENTS.md").exists();
        let manifest_path = src.join("docs").join("harness.toml");
        let has_manifest = manifest_path.exists();

        let mut name = None;
        let mut description = None;
        let mut work_type = None;

        if has_manifest {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = toml::from_str::<HarnessManifest>(&content) {
                    name = Some(manifest.name);
                    description = Some(manifest.description);
                    work_type = Some(manifest.work_type);
                }
            }
        }

        let mut found_files = Vec::new();
        self.collect_harness_files(src, src, &mut found_files)?;

        Ok(HarnessImportInspection {
            has_agents_md,
            has_manifest,
            name,
            description,
            work_type,
            found_files,
        })
    }

    fn collect_harness_files(
        &self,
        base_dir: &Path,
        current_dir: &Path,
        list: &mut Vec<String>,
    ) -> DomainResult<()> {
        let entries =
            fs::read_dir(current_dir).map_err(|e| DomainError::Database(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
            let path = entry.path();
            let name_os = entry.file_name();
            let name = name_os.to_string_lossy();
            if name == ".git" || name == ".DS_Store" {
                continue;
            }
            if path.is_dir() {
                self.collect_harness_files(base_dir, &path, list)?;
            } else {
                let rel_path = path
                    .strip_prefix(base_dir)
                    .map_err(|e| DomainError::Database(e.to_string()))?
                    .to_string_lossy()
                    .into_owned();
                list.push(rel_path);
            }
        }
        Ok(())
    }

    pub fn import_harness_from_folder(
        &self,
        source_path: &str,
        options: HarnessImportOptions,
    ) -> DomainResult<HarnessTemplateDetail> {
        let src = Path::new(source_path);
        if !src.exists() || !src.is_dir() {
            return Err(DomainError::Database(
                "Source folder does not exist or is not a directory".into(),
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let target_dir = self.harnesses_dir.join(&id);
        if target_dir.exists() {
            return Err(DomainError::Database(format!(
                "Harness template directory '{}' already exists",
                id
            )));
        }

        // Copy directory excluding .git / .DS_Store
        self.copy_harness_dir(src, &target_dir)?;

        // Ensure AGENTS.md exists
        if !target_dir.join("AGENTS.md").exists() {
            fs::write(
                target_dir.join("AGENTS.md"),
                "# Agent Workspace Instructions\n\nPrimary entrypoint.",
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;
        }

        // Generate / Update docs/harness.toml
        let manifest_path = target_dir.join("docs").join("harness.toml");
        let mut manifest = if manifest_path.exists() {
            if let Ok(toml_content) = fs::read_to_string(&manifest_path) {
                toml::from_str::<HarnessManifest>(&toml_content).unwrap_or_else(|_| {
                    self.fallback_manifest(
                        &id,
                        &options.name,
                        &options.description,
                        &options.work_type,
                        &target_dir,
                    )
                })
            } else {
                self.fallback_manifest(
                    &id,
                    &options.name,
                    &options.description,
                    &options.work_type,
                    &target_dir,
                )
            }
        } else {
            self.fallback_manifest(
                &id,
                &options.name,
                &options.description,
                &options.work_type,
                &target_dir,
            )
        };

        // Always update metadata based on user options input in the import wizard
        manifest.name = options.name.clone();
        manifest.description = options.description.clone();
        manifest.work_type = options.work_type.clone();

        let parent = manifest_path.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| DomainError::Database(e.to_string()))?;
        }
        let updated_toml = toml::to_string(&manifest)
            .map_err(|e| DomainError::Database(format!("Failed to format manifest: {e}")))?;
        fs::write(&manifest_path, updated_toml)
            .map_err(|e| DomainError::Database(e.to_string()))?;

        // Save index to DB
        let now = chrono::Utc::now().to_rfc3339();
        let summary = HarnessTemplateSummary {
            id: id.clone(),
            name: options.name,
            description: options.description,
            work_type: options.work_type,
            created_from_preset: None,
            selected_modules: manifest.selected_modules.clone(),
            source_type: "local".into(),
            source_path: Some(source_path.to_string()),
            created_at: now.clone(),
            updated_at: now,
            file_count: count_files_recursive(&target_dir).unwrap_or(0),
            has_agents_md: true,
            has_manifest: true,
            is_valid: true,
        };
        self.repo.save_harness(&summary)?;

        self.get_harness_template(&id)
    }

    fn fallback_manifest(
        &self,
        id: &str,
        name: &str,
        description: &str,
        work_type: &str,
        target_dir: &Path,
    ) -> HarnessManifest {
        let mut found_files = Vec::new();
        let _ = self.collect_harness_files(target_dir, target_dir, &mut found_files);

        let required_files = vec!["AGENTS.md".into(), "docs/harness.toml".into()];
        let mut files = Vec::new();
        for f in found_files {
            if f != "AGENTS.md" && f != "docs/harness.toml" {
                let kind = if f.ends_with(".json") {
                    "json"
                } else {
                    "markdown"
                };
                files.push(HarnessTemplateFile {
                    path: f,
                    kind: kind.into(),
                    standard: false,
                });
            }
        }

        HarnessManifest {
            id: id.to_string(),
            name: name.to_string(),
            version: "1.0.0".into(),
            description: description.to_string(),
            work_type: work_type.to_string(),
            created_from_preset: None,
            selected_modules: Vec::new(),
            source: "local".into(),
            required_files,
            files,
        }
    }

    fn copy_harness_dir(&self, src: &Path, dst: &Path) -> DomainResult<()> {
        fs::create_dir_all(dst).map_err(|e| DomainError::Database(e.to_string()))?;
        for entry in fs::read_dir(src).map_err(|e| DomainError::Database(e.to_string()))? {
            let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
            let ty = entry
                .file_type()
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == ".git" || name_str == ".DS_Store" {
                continue;
            }
            if ty.is_dir() {
                self.copy_harness_dir(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.join(entry.file_name()))
                    .map_err(|e| DomainError::Database(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub fn extract_harness_from_project(
        &self,
        project_id: &str,
        options: HarnessExtractOptions,
    ) -> DomainResult<HarnessTemplateDetail> {
        let project_path_str =
            self.project_repo
                .get_project_path(project_id)?
                .ok_or_else(|| {
                    DomainError::Database(format!("Project '{}' not found in database", project_id))
                })?;

        let project_path = Path::new(&project_path_str);
        if !project_path.exists() || !project_path.is_dir() {
            return Err(DomainError::Database(format!(
                "Project path '{}' does not exist on disk",
                project_path_str
            )));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let target_dir = self.harnesses_dir.join(&id);
        if target_dir.exists() {
            return Err(DomainError::Database(format!(
                "Harness template directory '{}' already exists",
                id
            )));
        }

        fs::create_dir_all(&target_dir).map_err(|e| DomainError::Database(e.to_string()))?;

        // Copy selected files
        for rel_path in &options.selected_files {
            let src_file = project_path.join(rel_path);
            let dst_file = target_dir.join(rel_path);

            if !src_file.exists() {
                continue;
            }

            let parent = dst_file.parent().unwrap();
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| DomainError::Database(e.to_string()))?;
            }

            fs::copy(&src_file, &dst_file).map_err(|e| DomainError::Database(e.to_string()))?;
        }

        // Ensure AGENTS.md exists
        if !target_dir.join("AGENTS.md").exists() {
            fs::write(
                target_dir.join("AGENTS.md"),
                "# Agent Workspace Instructions\n\nExtracted entrypoint.",
            )
            .map_err(|e| DomainError::Database(e.to_string()))?;
        }

        // Generate docs/harness.toml
        let required_files = vec!["AGENTS.md".into(), "docs/harness.toml".into()];
        let mut files = Vec::new();
        for f in &options.selected_files {
            if f != "AGENTS.md" && f != "docs/harness.toml" {
                let kind = if f.ends_with(".json") {
                    "json"
                } else {
                    "markdown"
                };
                files.push(HarnessTemplateFile {
                    path: f.clone(),
                    kind: kind.into(),
                    standard: true,
                });
            }
        }

        let manifest = HarnessManifest {
            id: id.clone(),
            name: options.name.clone(),
            version: "1.0.0".into(),
            description: options.description.clone(),
            work_type: options.work_type.clone(),
            created_from_preset: None,
            selected_modules: Vec::new(),
            source: "local".into(),
            required_files,
            files,
        };

        let manifest_path = target_dir.join("docs").join("harness.toml");
        let parent = manifest_path.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| DomainError::Database(e.to_string()))?;
        }
        let updated_toml = toml::to_string(&manifest)
            .map_err(|e| DomainError::Database(format!("Failed to format manifest: {e}")))?;
        fs::write(&manifest_path, updated_toml)
            .map_err(|e| DomainError::Database(e.to_string()))?;

        // Save index
        let now = chrono::Utc::now().to_rfc3339();
        let summary = HarnessTemplateSummary {
            id: id.clone(),
            name: options.name,
            description: options.description,
            work_type: options.work_type,
            created_from_preset: None,
            selected_modules: Vec::new(),
            source_type: "project".into(),
            source_path: Some(project_path_str),
            created_at: now.clone(),
            updated_at: now,
            file_count: count_files_recursive(&target_dir).unwrap_or(0),
            has_agents_md: true,
            has_manifest: true,
            is_valid: true,
        };
        self.repo.save_harness(&summary)?;

        self.get_harness_template(&id)
    }
}

fn count_files_recursive(dir: &Path) -> std::io::Result<usize> {
    let mut count = 0;
    if dir.exists() && dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name_os = entry.file_name();
            let name = name_os.to_string_lossy();
            if name == ".git" || name == ".DS_Store" {
                continue;
            }
            if path.is_dir() {
                count += count_files_recursive(&path)?;
            } else {
                count += 1;
            }
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::DomainResult;
    use crate::domain::project::Project;
    use crate::domain::skill::{Category, SkillPackageRecord, UserSkillMeta};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;

    struct MockSkillRepo;
    impl SkillRepository for MockSkillRepo {
        fn get_projects(&self) -> DomainResult<Vec<Project>> {
            Ok(Vec::new())
        }
        fn get_project_path(&self, id: &str) -> DomainResult<Option<String>> {
            if id == "proj-1" {
                Ok(Some("/tmp/mock-proj".into()))
            } else {
                Ok(None)
            }
        }
        fn create_project(&self, _project: &Project) -> DomainResult<()> {
            Ok(())
        }
        fn delete_project(&self, _id: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_user_meta(&self, _skill_id: &str) -> DomainResult<Option<UserSkillMeta>> {
            Ok(None)
        }
        fn save_user_meta(
            &self,
            _skill_id: &str,
            _category_id: Option<&str>,
            _user_notes: Option<&str>,
        ) -> DomainResult<()> {
            Ok(())
        }
        fn delete_user_meta(&self, _skill_id: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_skill_package(&self, _skill_id: &str) -> DomainResult<Option<SkillPackageRecord>> {
            Ok(None)
        }
        fn save_skill_package(&self, _record: &SkillPackageRecord) -> DomainResult<()> {
            Ok(())
        }
        fn find_skill_by_source(&self, _source_url: &str) -> DomainResult<Option<String>> {
            Ok(None)
        }
        fn get_project_skills(&self, _project_id: &str) -> DomainResult<Vec<String>> {
            Ok(Vec::new())
        }
        fn save_project_skill(
            &self,
            _project_id: &str,
            _skill_id: &str,
            _enabled: bool,
        ) -> DomainResult<()> {
            Ok(())
        }
        fn get_projects_using_skill(&self, _skill_id: &str) -> DomainResult<Vec<String>> {
            Ok(Vec::new())
        }
        fn save_project_skill_state(
            &self,
            _project_id: &str,
            _skill_id: &str,
            _installed_commit: Option<&str>,
            _sync_state: &str,
        ) -> DomainResult<()> {
            Ok(())
        }
        fn get_categories(&self) -> DomainResult<Vec<Category>> {
            Ok(Vec::new())
        }
        fn create_category(
            &self,
            _id: &str,
            _name: &str,
            _created_at: &str,
        ) -> DomainResult<Category> {
            Ok(Category {
                id: "".into(),
                name: "".into(),
                created_at: "".into(),
            })
        }
        fn rename_category(&self, _id: &str, _name: &str) -> DomainResult<()> {
            Ok(())
        }
        fn delete_category(&self, _id: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_custom_description(&self, _target_id: &str) -> DomainResult<Option<String>> {
            Ok(None)
        }
        fn save_custom_description(
            &self,
            _target_id: &str,
            _target_kind: &str,
            _custom_description: &str,
        ) -> DomainResult<()> {
            Ok(())
        }
        fn delete_custom_description(&self, _target_id: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_all_custom_descriptions(
            &self,
        ) -> DomainResult<Vec<crate::domain::skill::SkillDescriptionRecord>> {
            Ok(Vec::new())
        }
        fn import_custom_descriptions(
            &self,
            _records: Vec<crate::domain::skill::SkillDescriptionRecord>,
            _conflict_strategy: &str,
        ) -> DomainResult<()> {
            Ok(())
        }
        fn delete_descriptions(&self, _target_ids: &[String]) -> DomainResult<()> {
            Ok(())
        }
        fn migrate_git_skill_id(&self, _old_id: &str, _new_id: &str) -> DomainResult<()> {
            Ok(())
        }
    }

    use std::sync::Mutex;
    struct MockHarnessRepo {
        items: Mutex<Vec<HarnessTemplateSummary>>,
    }
    impl HarnessRepository for MockHarnessRepo {
        fn get_harnesses(&self) -> DomainResult<Vec<HarnessTemplateSummary>> {
            Ok(self.items.lock().unwrap().clone())
        }
        fn save_harness(&self, summary: &HarnessTemplateSummary) -> DomainResult<()> {
            let mut list = self.items.lock().unwrap();
            list.retain(|i| i.id != summary.id);
            list.push(summary.clone());
            Ok(())
        }
        fn delete_harness(&self, id: &str) -> DomainResult<()> {
            let mut list = self.items.lock().unwrap();
            list.retain(|i| i.id != id);
            Ok(())
        }
    }

    struct TempFixture(PathBuf);
    impl TempFixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("harness-test-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TempFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_create_and_get_harness_template() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let input = CreateHarnessTemplateInput {
            name: "My Template".into(),
            description: "A description".into(),
            work_type: "code".into(),
            preset_id: None,
            selected_modules: vec!["technical-design".into(), "feature-development".into()],
            optional_files: vec![
                "docs/decision-record.md".into(),
                "docs/feature_list.json".into(),
            ],
        };

        let detail = service.create_harness_template(input).unwrap();
        assert!(uuid::Uuid::parse_str(&detail.id).is_ok());
        assert_eq!(detail.name, "My Template");
        assert_eq!(
            detail.selected_modules,
            vec![
                "technical-design".to_string(),
                "feature-development".to_string(),
            ]
        );
        assert_eq!(detail.files.len(), 4);
        assert!(detail.validation.is_valid);
        let agents = service.read_harness_file(&detail.id, "AGENTS.md").unwrap();
        assert!(agents.content.contains("docs/decision-record.md"));
        assert!(agents.content.contains("docs/feature_list.json"));

        let manifest_content = service
            .read_harness_file(&detail.id, "docs/harness.toml")
            .unwrap()
            .content;
        let manifest: HarnessManifest = toml::from_str(&manifest_content).unwrap();
        assert_eq!(manifest.selected_modules, detail.selected_modules);
        assert!(manifest.created_from_preset.is_none());

        let list = service.get_harness_templates().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, detail.id);
        assert_eq!(list[0].selected_modules, detail.selected_modules);
    }

    #[test]
    fn rejects_preset_from_a_different_work_type() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let error = service
            .create_harness_template(CreateHarnessTemplateInput {
                name: "Invalid".into(),
                description: "".into(),
                work_type: "presentation".into(),
                preset_id: Some("document-academic-paper".into()),
                selected_modules: vec![],
                optional_files: vec![],
            })
            .unwrap_err();

        assert!(error.to_string().contains("belongs to work type"));
    }

    #[test]
    fn custom_work_does_not_accept_a_system_preset() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let error = service
            .create_harness_template(CreateHarnessTemplateInput {
                name: "Invalid Custom".into(),
                description: "".into(),
                work_type: "custom".into(),
                preset_id: Some("document-academic-paper".into()),
                selected_modules: vec![],
                optional_files: vec![],
            })
            .unwrap_err();

        assert!(error.to_string().contains("Custom Work cannot use"));
    }

    #[test]
    fn test_edit_file() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let input = CreateHarnessTemplateInput {
            name: "T1".into(),
            description: "".into(),
            work_type: "custom".into(),
            preset_id: None,
            selected_modules: vec![],
            optional_files: vec![],
        };
        let template_id = service.create_harness_template(input).unwrap().id;

        // Create new file
        let new_file = service
            .create_harness_file(&template_id, "docs/new-file.md", "markdown")
            .unwrap();
        assert_eq!(new_file.path, "docs/new-file.md");

        // Write
        service
            .write_harness_file(&template_id, "docs/new-file.md", "New Content")
            .unwrap();

        // Read
        let read = service
            .read_harness_file(&template_id, "docs/new-file.md")
            .unwrap();
        assert_eq!(read.content, "New Content");

        // Delete
        service
            .delete_harness_file(&template_id, "docs/new-file.md")
            .unwrap();
        assert!(service
            .read_harness_file(&template_id, "docs/new-file.md")
            .is_err());
    }

    fn all_selected_code_paths() -> Vec<String> {
        vec![
            "docs/architecture.md".into(),
            "docs/task-status.md".into(),
            "docs/session-handoff.md".into(),
            "docs/verification.md".into(),
            "docs/risk-rules.md".into(),
            "docs/decision-record.md".into(),
            "docs/feature_list.json".into(),
            "docs/review-rubric.md".into(),
            "docs/review-findings.md".into(),
        ]
    }

    #[test]
    fn creates_a_composed_code_harness_with_deduplicated_shared_files() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let detail = service
            .create_harness_template(CreateHarnessTemplateInput {
                name: "Full delivery".into(),
                description: "".into(),
                work_type: "code".into(),
                selected_modules: vec![
                    "technical-design".into(),
                    "feature-development".into(),
                    "code-review".into(),
                ],
                preset_id: None,
                optional_files: all_selected_code_paths(),
            })
            .unwrap();

        assert_eq!(
            detail
                .files
                .iter()
                .filter(|file| file.path == "docs/task-status.md")
                .count(),
            1
        );
        assert!(detail
            .files
            .iter()
            .any(|file| file.path == "docs/decision-record.md"));
        assert!(detail
            .files
            .iter()
            .any(|file| file.path == "docs/feature_list.json"));
        assert!(detail
            .files
            .iter()
            .any(|file| file.path == "docs/review-findings.md"));
    }

    #[test]
    fn combined_agents_file_routes_tasks_and_orders_selected_roles() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let detail = service
            .create_harness_template(CreateHarnessTemplateInput {
                name: "Full delivery".into(),
                description: "".into(),
                work_type: "code".into(),
                selected_modules: vec![
                    "technical-design".into(),
                    "feature-development".into(),
                    "code-review".into(),
                ],
                preset_id: None,
                optional_files: all_selected_code_paths(),
            })
            .unwrap();

        let content = service
            .read_harness_file(&detail.id, "AGENTS.md")
            .unwrap()
            .content;

        assert!(content.contains("## Task Classification"));
        assert!(content.contains("## Technical Design Role"));
        assert!(content.contains("## Feature Development Role"));
        assert!(content.contains("## Code Review Role"));
        assert!(content.contains("Technical Design -> Feature Development -> Code Review"));
    }

    #[test]
    fn code_work_rejects_empty_modules() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let error = service
            .create_harness_template(CreateHarnessTemplateInput {
                name: "Empty modules".into(),
                description: "".into(),
                work_type: "code".into(),
                preset_id: None,
                selected_modules: vec![],
                optional_files: vec![],
            })
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Code Work requires at least one module")
                || error.to_string().contains("empty")
        );
    }

    #[test]
    fn code_work_rejects_duplicate_modules() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let error = service
            .create_harness_template(CreateHarnessTemplateInput {
                name: "Dup modules".into(),
                description: "".into(),
                work_type: "code".into(),
                preset_id: None,
                selected_modules: vec!["technical-design".into(), "technical-design".into()],
                optional_files: vec![],
            })
            .unwrap_err();

        assert!(
            error.to_string().contains("Duplicate module ID")
                || error.to_string().contains("duplicate")
        );
    }

    #[test]
    fn code_work_rejects_unknown_module() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let error = service
            .create_harness_template(CreateHarnessTemplateInput {
                name: "Unknown module".into(),
                description: "".into(),
                work_type: "code".into(),
                preset_id: None,
                selected_modules: vec!["nonexistent".into()],
                optional_files: vec![],
            })
            .unwrap_err();

        assert!(
            error.to_string().contains("Unknown module ID")
                || error.to_string().contains("unknown")
        );
    }

    #[test]
    fn code_work_rejects_preset_id() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let error = service
            .create_harness_template(CreateHarnessTemplateInput {
                name: "Preset with code".into(),
                description: "".into(),
                work_type: "code".into(),
                preset_id: Some("document-academic-paper".into()),
                selected_modules: vec!["technical-design".into()],
                optional_files: vec![],
            })
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Code Work cannot use a built-in preset")
                || error.to_string().contains("preset")
        );
    }

    #[test]
    fn document_work_rejects_modules() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        let error = service
            .create_harness_template(CreateHarnessTemplateInput {
                name: "Doc with modules".into(),
                description: "".into(),
                work_type: "document".into(),
                preset_id: Some("document-academic-paper".into()),
                selected_modules: vec!["technical-design".into()],
                optional_files: vec![],
            })
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Cannot select modules for document work")
                || error.to_string().contains("modules")
        );
    }

    #[test]
    fn test_legacy_import_without_modules() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        // Create a fake legacy folder with AGENTS.md and docs/harness.toml
        let legacy_src = fixture.0.join("legacy_src");
        fs::create_dir_all(&legacy_src).unwrap();
        fs::create_dir_all(legacy_src.join("docs")).unwrap();
        fs::write(
            legacy_src.join("AGENTS.md"),
            "# Agent Workspace Instructions\n\nLegacy agents",
        )
        .unwrap();
        fs::write(
            legacy_src.join("docs").join("harness.toml"),
            r#"id = "legacy-id"
name = "Legacy Template"
version = "1.0.0"
description = "Legacy test template"
work_type = "code"
selected_modules = []
source = "local"
required_files = ["AGENTS.md", "docs/harness.toml"]
files = []
"#,
        )
        .unwrap();

        let options = HarnessImportOptions {
            name: "Legacy Imported Template".into(),
            description: "A legacy imported description".into(),
            work_type: "code".into(),
            preset_id: None,
        };

        let detail = service
            .import_harness_from_folder(legacy_src.to_str().unwrap(), options)
            .unwrap();
        assert_eq!(detail.selected_modules.len(), 0);
        assert!(detail.validation.is_valid); // It is a legacy template, so it should be valid
        assert!(
            detail
                .validation
                .warnings
                .iter()
                .any(|w| w.to_lowercase().contains("legacy")),
            "Expected legacy warning when selected_modules is empty, got warnings: {:?}",
            detail.validation.warnings
        );
    }

    #[test]
    fn test_validation_warning_missing_module_file_and_heading() {
        let fixture = TempFixture::new();
        let repo = Arc::new(MockHarnessRepo {
            items: Mutex::new(Vec::new()),
        });
        let proj_repo = Arc::new(MockSkillRepo);
        let service = HarnessService::with_harnesses_dir(repo, proj_repo, fixture.0.clone());

        // Create a composed Code template
        let input = CreateHarnessTemplateInput {
            name: "Composed Code Template".into(),
            description: "Test description".into(),
            work_type: "code".into(),
            preset_id: None,
            selected_modules: vec!["code-review".into()],
            optional_files: vec![
                "docs/review-rubric.md".into(),
                "docs/review-findings.md".into(),
            ],
        };

        let detail = service.create_harness_template(input).unwrap();
        assert!(detail.validation.is_valid);
        assert!(detail.validation.warnings.is_empty());

        // Now remove docs/review-findings.md
        let findings_path = fixture.0.join(&detail.id).join("docs/review-findings.md");
        fs::remove_file(findings_path).unwrap();

        // Run validation
        let report = service.validate_harness_template(&detail.id).unwrap();

        // Assert the warning exists, contains "code-review", and contains the missing path "docs/review-findings.md"
        let mut found_warning = false;
        for warning in &report.warnings {
            if warning.contains("code-review") && warning.contains("docs/review-findings.md") {
                found_warning = true;
            }
        }
        assert!(found_warning, "Expected warning about missing docs/review-findings.md for code-review module, got: {:?}", report.warnings);

        // Modify AGENTS.md to remove the heading "## Code Review Role" and the path "docs/review-rubric.md"
        let agents_path = fixture.0.join(&detail.id).join("AGENTS.md");
        let agents_content = fs::read_to_string(&agents_path).unwrap();
        let updated_content = agents_content
            .replace("## Code Review Role", "")
            .replace("docs/review-rubric.md", "");
        fs::write(&agents_path, updated_content).unwrap();

        // Run validation again
        let report2 = service.validate_harness_template(&detail.id).unwrap();

        // Assert warning about missing heading "## Code Review Role"
        let found_heading_warning = report2
            .warnings
            .iter()
            .any(|w| w.contains("heading") && w.contains("Code Review Role"));
        assert!(
            found_heading_warning,
            "Expected warning about missing heading '## Code Review Role', got: {:?}",
            report2.warnings
        );

        // Assert warning about missing dedicated path reference
        let found_path_warning = report2
            .warnings
            .iter()
            .any(|w| w.contains("reference") && w.contains("docs/review-rubric.md"));
        assert!(
            found_path_warning,
            "Expected warning about missing reference to 'docs/review-rubric.md', got: {:?}",
            report2.warnings
        );
    }
}
