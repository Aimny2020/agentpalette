use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::harness::HarnessManifest;
use crate::domain::ports::{HarnessRepository, ProjectHarnessRepository, SkillRepository};
use crate::domain::project_harness::{
    ProjectHarnessApplicationPreview, ProjectHarnessAppliedFile, ProjectHarnessApplyInput,
    ProjectHarnessConflict, ProjectHarnessFile, ProjectHarnessRecord, ProjectHarnessStatus,
};

pub struct ProjectHarnessService {
    project_repo: Arc<dyn SkillRepository>,
    template_repo: Arc<dyn HarnessRepository>,
    project_harness_repo: Arc<dyn ProjectHarnessRepository>,
    templates_dir: PathBuf,
}

impl ProjectHarnessService {
    pub fn new(
        project_repo: Arc<dyn SkillRepository>,
        template_repo: Arc<dyn HarnessRepository>,
        project_harness_repo: Arc<dyn ProjectHarnessRepository>,
    ) -> Self {
        Self {
            project_repo,
            template_repo,
            project_harness_repo,
            templates_dir: dirs::home_dir()
                .expect("Failed to locate home directory")
                .join(".agent-forge")
                .join("harnesses"),
        }
    }

    #[cfg(test)]
    fn with_dirs(
        project_repo: Arc<dyn SkillRepository>,
        template_repo: Arc<dyn HarnessRepository>,
        project_harness_repo: Arc<dyn ProjectHarnessRepository>,
        templates_dir: PathBuf,
    ) -> Self {
        Self {
            project_repo,
            template_repo,
            project_harness_repo,
            templates_dir,
        }
    }

    pub fn get_status(&self, project_id: &str) -> DomainResult<ProjectHarnessStatus> {
        let project_path = self.project_path(project_id)?;
        let record = self.project_harness_repo.get_project_harness(project_id)?;
        let files = list_project_files(&project_path, record.as_ref())?;
        let has_agents_md = project_path.join("AGENTS.md").is_file();
        let manifest_path = project_path.join("docs/harness.toml");
        let manifest_parseable = fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|content| toml::from_str::<toml::Value>(&content).ok())
            .is_some();
        let source_status = record
            .as_ref()
            .map(|r| self.source_status(r))
            .unwrap_or_else(|| "unknown".into());
        let mut warnings = Vec::new();
        if !has_agents_md {
            warnings.push("项目根目录缺少 AGENTS.md".into());
        }
        if !manifest_parseable {
            warnings.push("docs/harness.toml 不存在或无法解析".into());
        }
        let state = match record.as_ref() {
            Some(_) if has_agents_md && manifest_parseable => "managed",
            Some(_) => "invalid",
            None if has_agents_md && manifest_parseable => "unmanaged_detected",
            None => "absent",
        };
        Ok(ProjectHarnessStatus {
            project_id: project_id.into(),
            state: state.into(),
            source_template_id: record.as_ref().and_then(|r| r.source_template_id.clone()),
            source_template_hash: record.as_ref().and_then(|r| r.source_template_hash.clone()),
            applied_at: record.as_ref().map(|r| r.applied_at.clone()),
            source_status,
            has_agents_md,
            manifest_parseable,
            files,
            warnings,
        })
    }

    pub fn preview_application(
        &self,
        project_id: &str,
        template_id: &str,
    ) -> DomainResult<ProjectHarnessApplicationPreview> {
        if self
            .project_harness_repo
            .get_project_harness(project_id)?
            .is_some()
        {
            return Err(DomainError::Database(
                "项目已有 Harness，请先删除当前 Harness".into(),
            ));
        }
        let project_path = self.project_path(project_id)?;
        let template_path = self.template_path(template_id)?;
        let template_files = collect_files(&template_path)?;
        let mut conflicts = Vec::new();
        for file in &template_files {
            let project_file = project_path.join(&file.path);
            if project_file.is_file() {
                conflicts.push(ProjectHarnessConflict {
                    path: file.path.clone(),
                    template_content: file.content.clone(),
                    project_content: fs::read_to_string(project_file).ok(),
                });
            }
        }
        let references = agents_references(
            template_files
                .iter()
                .find(|file| file.path == "AGENTS.md")
                .map(|file| file.content.as_str())
                .unwrap_or_default(),
        );
        let template_paths: HashSet<String> =
            template_files.iter().map(|f| f.path.clone()).collect();
        let missing_agents_references = references
            .iter()
            .filter(|path| !template_paths.contains(*path))
            .cloned()
            .collect();
        Ok(ProjectHarnessApplicationPreview {
            project_id: project_id.into(),
            template_id: template_id.into(),
            conflicts,
            template_files,
            final_agents_references: references,
            missing_agents_references,
        })
    }

    pub fn apply(&self, input: ProjectHarnessApplyInput) -> DomainResult<ProjectHarnessStatus> {
        if self
            .project_harness_repo
            .get_project_harness(&input.project_id)?
            .is_some()
        {
            return Err(DomainError::Database(
                "项目已有 Harness，请先删除当前 Harness".into(),
            ));
        }
        let preview = self.preview_application(&input.project_id, &input.template_id)?;
        if !preview.missing_agents_references.is_empty() {
            return Err(DomainError::Database(
                "AGENTS.md 引用了模板中不存在的文件".into(),
            ));
        }
        let decisions: HashMap<String, String> = input
            .decisions
            .into_iter()
            .map(|d| (d.path, d.action))
            .collect();
        for conflict in &preview.conflicts {
            if !matches!(
                decisions.get(&conflict.path).map(String::as_str),
                Some("keep" | "overwrite" | "skip")
            ) {
                return Err(DomainError::Database(format!(
                    "未处理文件冲突 '{}'",
                    conflict.path
                )));
            }
        }
        let project_path = self.project_path(&input.project_id)?;
        let mut writes: Vec<(PathBuf, Vec<u8>, Option<Vec<u8>>)> = Vec::new();
        for file in &preview.template_files {
            let destination = project_path.join(&file.path);
            let action = decisions.get(&file.path).map(String::as_str);
            if action == Some("keep") || action == Some("skip") {
                continue;
            }
            let mut content = file.content.clone();
            if file.path == "docs/harness.toml" {
                content = self.project_manifest(&file.content)?;
            }
            let old = fs::read(&destination).ok();
            writes.push((destination, content.into_bytes(), old));
        }
        let mut completed: Vec<(PathBuf, Vec<u8>, Option<Vec<u8>>)> = Vec::new();
        for (destination, content, old) in &writes {
            if old.is_some() {
                backup_file(&input.project_id, destination, old.as_ref().unwrap())?;
            }
            if let Err(error) = write_file(destination, content) {
                for (rollback_path, _, rollback_old) in completed.iter().rev() {
                    restore_file(rollback_path, rollback_old.as_ref());
                }
                return Err(error);
            }
            completed.push((destination.clone(), content.clone(), old.clone()));
        }
        let applied_files = collect_files(&project_path)?
            .into_iter()
            .filter(|file| file.path == "AGENTS.md" || file.path.starts_with("docs/"))
            .map(|file| ProjectHarnessAppliedFile {
                path: file.path,
                applied_content_hash: content_hash(file.content.as_bytes()),
                created_by_application: true,
            })
            .collect();
        let record = ProjectHarnessRecord {
            project_id: input.project_id.clone(),
            source_template_id: Some(input.template_id.clone()),
            source_template_hash: Some(template_hash(&preview.template_files)),
            applied_at: chrono::Utc::now().to_rfc3339(),
            managed_state: "managed".into(),
            applied_files,
        };
        self.project_harness_repo.save_project_harness(&record)?;
        self.get_status(&input.project_id)
    }

    pub fn read_file(&self, project_id: &str, path: &str) -> DomainResult<ProjectHarnessFile> {
        let project_path = self.project_path(project_id)?;
        validate_project_path(path)?;
        let full_path = project_path.join(path);
        let content =
            fs::read_to_string(&full_path).map_err(|e| DomainError::Database(e.to_string()))?;
        let record = self.project_harness_repo.get_project_harness(project_id)?;
        Ok(project_file(path, content, record.as_ref()))
    }

    pub fn write_file(
        &self,
        project_id: &str,
        path: &str,
        content: &str,
    ) -> DomainResult<ProjectHarnessFile> {
        validate_project_path(path)?;
        let project_path = self.project_path(project_id)?;
        write_file(&project_path.join(path), content.as_bytes())?;
        let record = self.project_harness_repo.get_project_harness(project_id)?;
        Ok(project_file(path, content.into(), record.as_ref()))
    }

    pub fn unmanage(&self, project_id: &str) -> DomainResult<()> {
        self.project_harness_repo.delete_project_harness(project_id)
    }

    pub fn adopt(&self, project_id: &str) -> DomainResult<ProjectHarnessStatus> {
        let project_path = self.project_path(project_id)?;
        let files = collect_files(&project_path)?;
        let record = ProjectHarnessRecord {
            project_id: project_id.into(),
            source_template_id: None,
            source_template_hash: None,
            applied_at: chrono::Utc::now().to_rfc3339(),
            managed_state: "unmanaged-adopted".into(),
            applied_files: files
                .into_iter()
                .map(|file| ProjectHarnessAppliedFile {
                    path: file.path,
                    applied_content_hash: content_hash(file.content.as_bytes()),
                    created_by_application: false,
                })
                .collect(),
        };
        self.project_harness_repo.save_project_harness(&record)?;
        self.get_status(project_id)
    }

    pub fn create_file(&self, project_id: &str, path: &str) -> DomainResult<ProjectHarnessFile> {
        validate_project_path(path)?;
        let project_path = self.project_path(project_id)?;
        let full_path = project_path.join(path);
        if full_path.exists() {
            return Err(DomainError::Database("Harness 文件已存在".into()));
        }
        write_file(&full_path, b"")?;
        let record = self.project_harness_repo.get_project_harness(project_id)?;
        Ok(project_file(path, String::new(), record.as_ref()))
    }

    pub fn delete_file(
        &self,
        project_id: &str,
        path: &str,
        explicit_confirmation: bool,
    ) -> DomainResult<()> {
        validate_project_path(path)?;
        let record = self
            .project_harness_repo
            .get_project_harness(project_id)?
            .ok_or_else(|| DomainError::Database("项目尚未纳管 Harness".into()))?;
        let project_path = self.project_path(project_id)?;
        let full_path = project_path.join(path);
        let current = fs::read(&full_path).map_err(|e| DomainError::Database(e.to_string()))?;
        let applied = record.applied_files.iter().find(|file| file.path == path);
        if !explicit_confirmation {
            return Err(DomainError::Database(
                "删除 Harness 文件必须经过明确确认".into(),
            ));
        }
        if let Some(applied) = applied {
            if content_hash(&current) != applied.applied_content_hash {
                return Err(DomainError::Database("文件已被修改，不能静默删除".into()));
            }
        } else if path == "AGENTS.md" || path == "docs/harness.toml" {
            return Err(DomainError::Database("文件已被修改，不能静默删除".into()));
        }
        fs::remove_file(full_path).map_err(|e| DomainError::Database(e.to_string()))
    }

    fn project_path(&self, project_id: &str) -> DomainResult<PathBuf> {
        let path = self
            .project_repo
            .get_project_path(project_id)?
            .ok_or_else(|| DomainError::Database(format!("Project '{}' not found", project_id)))?;
        let path = PathBuf::from(path);
        if !path.is_dir() {
            return Err(DomainError::Database("项目目录不存在".into()));
        }
        Ok(path)
    }

    fn template_path(&self, template_id: &str) -> DomainResult<PathBuf> {
        let path = self.templates_dir.join(template_id);
        if !path.is_dir()
            || self
                .template_repo
                .get_harnesses()?
                .iter()
                .all(|h| h.id != template_id)
        {
            return Err(DomainError::Database("Harness 模板不存在".into()));
        }
        Ok(path)
    }

    fn project_manifest(&self, content: &str) -> DomainResult<String> {
        let mut manifest: HarnessManifest = toml::from_str(content)
            .map_err(|e| DomainError::Database(format!("Harness 配置无法解析: {e}")))?;
        manifest.id = uuid::Uuid::new_v4().to_string();
        manifest.source = "project".into();
        manifest.created_from_preset = None;
        toml::to_string(&manifest).map_err(|e| DomainError::Database(e.to_string()))
    }

    fn source_status(&self, record: &ProjectHarnessRecord) -> String {
        let Some(id) = record.source_template_id.as_deref() else {
            return "unknown".into();
        };
        let Ok(path) = self.template_path(id) else {
            return "deleted".into();
        };
        let Ok(files) = collect_files(&path) else {
            return "unknown".into();
        };
        if record.source_template_hash.as_deref() == Some(template_hash(&files).as_str()) {
            "available".into()
        } else {
            "changed".into()
        }
    }
}

fn validate_project_path(path: &str) -> DomainResult<()> {
    let clean = Path::new(path);
    if clean.is_absolute()
        || path.contains("..")
        || !(path == "AGENTS.md" || path.starts_with("docs/"))
    {
        return Err(DomainError::Database(
            "Harness 文件路径必须位于 AGENTS.md 或 docs/ 下".into(),
        ));
    }
    Ok(())
}

fn write_file(path: &Path, content: &[u8]) -> DomainResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| DomainError::Database(e.to_string()))?;
    }
    fs::write(path, content).map_err(|e| DomainError::Database(e.to_string()))
}

fn backup_file(project_id: &str, source: &Path, content: &[u8]) -> DomainResult<()> {
    let relative = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let backup_root = dirs::home_dir()
        .ok_or_else(|| DomainError::Database("无法定位本地备份目录".into()))?
        .join(".agent-forge")
        .join("project-harness-backups")
        .join(project_id)
        .join(
            chrono::Utc::now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
                .to_string(),
        );
    fs::create_dir_all(&backup_root).map_err(|e| DomainError::Database(e.to_string()))?;
    fs::write(backup_root.join(relative), content).map_err(|e| DomainError::Database(e.to_string()))
}

fn restore_file(path: &Path, old: Option<&Vec<u8>>) {
    if let Some(content) = old {
        let _ = fs::write(path, content);
    } else {
        let _ = fs::remove_file(path);
    }
}

fn collect_files(root: &Path) -> DomainResult<Vec<ProjectHarnessFile>> {
    let mut files = Vec::new();
    collect_files_inner(root, root, &mut files)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn collect_files_inner(
    root: &Path,
    current: &Path,
    files: &mut Vec<ProjectHarnessFile>,
) -> DomainResult<()> {
    if !current.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(current).map_err(|e| DomainError::Database(e.to_string()))? {
        let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == ".git" || name == ".DS_Store" || name == ".agentforge" {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_files_inner(root, &path, files)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| DomainError::Database(e.to_string()))?;
            let relative = relative.to_string_lossy().replace('\\', "/");
            if relative == "AGENTS.md" || relative.starts_with("docs/") {
                let content =
                    fs::read_to_string(&path).map_err(|e| DomainError::Database(e.to_string()))?;
                files.push(ProjectHarnessFile {
                    path: relative,
                    content,
                    exists: true,
                    changed_since_apply: false,
                    deletion_eligible: false,
                });
            }
        }
    }
    Ok(())
}

fn list_project_files(
    root: &Path,
    record: Option<&ProjectHarnessRecord>,
) -> DomainResult<Vec<ProjectHarnessFile>> {
    let mut files = collect_files(root)?;
    for file in &mut files {
        if let Some(record) = record {
            if let Some(applied) = record
                .applied_files
                .iter()
                .find(|item| item.path == file.path)
            {
                file.changed_since_apply =
                    content_hash(file.content.as_bytes()) != applied.applied_content_hash;
                file.deletion_eligible = !file.changed_since_apply;
            }
        }
    }
    Ok(files)
}

fn project_file(
    path: &str,
    content: String,
    record: Option<&ProjectHarnessRecord>,
) -> ProjectHarnessFile {
    let applied = record.and_then(|r| r.applied_files.iter().find(|f| f.path == path));
    let changed = applied
        .map(|f| content_hash(content.as_bytes()) != f.applied_content_hash)
        .unwrap_or(false);
    ProjectHarnessFile {
        path: path.into(),
        content,
        exists: true,
        changed_since_apply: changed,
        deletion_eligible: applied.is_some() && !changed,
    }
}

fn agents_references(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let start = line.find("docs/")?;
            let value = &line[start..];
            let end = value
                .find(|c: char| c == '`' || c.is_whitespace() || c == ')' || c == '，')
                .unwrap_or(value.len());
            let path = &value[..end];
            if path.ends_with(".md") || path.ends_with(".json") || path.ends_with(".toml") {
                Some(path.into())
            } else {
                None
            }
        })
        .collect()
}

fn content_hash(content: &[u8]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn template_hash(files: &[ProjectHarnessFile]) -> String {
    let mut value = String::new();
    for file in files {
        value.push_str(&file.path);
        value.push('\0');
        value.push_str(&file.content);
        value.push('\0');
    }
    content_hash(value.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::DomainResult;
    use crate::domain::harness::HarnessTemplateSummary;
    use crate::domain::ports::{HarnessRepository, ProjectHarnessRepository};
    use crate::domain::project::Project;
    use std::sync::Mutex;

    struct Projects {
        path: String,
    }
    impl SkillRepository for Projects {
        fn get_projects(&self) -> DomainResult<Vec<Project>> {
            Ok(Vec::new())
        }
        fn get_project_path(&self, id: &str) -> DomainResult<Option<String>> {
            Ok((id == "p1").then(|| self.path.clone()))
        }
        fn create_project(&self, _: &Project) -> DomainResult<()> {
            Ok(())
        }
        fn delete_project(&self, _: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_user_meta(
            &self,
            _: &str,
        ) -> DomainResult<Option<crate::domain::skill::UserSkillMeta>> {
            Ok(None)
        }
        fn save_user_meta(&self, _: &str, _: Option<&str>, _: Option<&str>) -> DomainResult<()> {
            Ok(())
        }
        fn delete_user_meta(&self, _: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_skill_package(
            &self,
            _: &str,
        ) -> DomainResult<Option<crate::domain::skill::SkillPackageRecord>> {
            Ok(None)
        }
        fn save_skill_package(
            &self,
            _: &crate::domain::skill::SkillPackageRecord,
        ) -> DomainResult<()> {
            Ok(())
        }
        fn find_skill_by_source(&self, _: &str) -> DomainResult<Option<String>> {
            Ok(None)
        }
        fn migrate_git_skill_id(&self, _: &str, _: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_project_skills(&self, _: &str) -> DomainResult<Vec<String>> {
            Ok(Vec::new())
        }
        fn save_project_skill(&self, _: &str, _: &str, _: bool) -> DomainResult<()> {
            Ok(())
        }
        fn get_projects_using_skill(&self, _: &str) -> DomainResult<Vec<String>> {
            Ok(Vec::new())
        }
        fn save_project_skill_state(
            &self,
            _: &str,
            _: &str,
            _: Option<&str>,
            _: &str,
        ) -> DomainResult<()> {
            Ok(())
        }
        fn get_categories(&self) -> DomainResult<Vec<crate::domain::skill::Category>> {
            Ok(Vec::new())
        }
        fn create_category(
            &self,
            _: &str,
            _: &str,
            _: &str,
        ) -> DomainResult<crate::domain::skill::Category> {
            unreachable!()
        }
        fn rename_category(&self, _: &str, _: &str) -> DomainResult<()> {
            Ok(())
        }
        fn delete_category(&self, _: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_custom_description(&self, _: &str) -> DomainResult<Option<String>> {
            Ok(None)
        }
        fn save_custom_description(&self, _: &str, _: &str, _: &str) -> DomainResult<()> {
            Ok(())
        }
        fn delete_custom_description(&self, _: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_all_custom_descriptions(
            &self,
        ) -> DomainResult<Vec<crate::domain::skill::SkillDescriptionRecord>> {
            Ok(Vec::new())
        }
        fn import_custom_descriptions(
            &self,
            _: Vec<crate::domain::skill::SkillDescriptionRecord>,
            _: &str,
        ) -> DomainResult<()> {
            Ok(())
        }
        fn delete_descriptions(&self, _: &[String]) -> DomainResult<()> {
            Ok(())
        }
    }

    struct Templates {
        id: String,
    }
    impl HarnessRepository for Templates {
        fn get_harnesses(&self) -> DomainResult<Vec<HarnessTemplateSummary>> {
            Ok(vec![HarnessTemplateSummary {
                id: self.id.clone(),
                name: "T".into(),
                description: String::new(),
                work_type: "code".into(),
                language: "en".into(),
                created_from_preset: None,
                selected_modules: Vec::new(),
                source_type: "local".into(),
                source_path: None,
                created_at: String::new(),
                updated_at: String::new(),
                file_count: 2,
                has_agents_md: true,
                has_manifest: true,
                is_valid: true,
            }])
        }
        fn save_harness(&self, _: &HarnessTemplateSummary) -> DomainResult<()> {
            Ok(())
        }
        fn delete_harness(&self, _: &str) -> DomainResult<()> {
            Ok(())
        }
    }

    struct ProjectHarnesses(Mutex<Option<ProjectHarnessRecord>>);
    impl ProjectHarnessRepository for ProjectHarnesses {
        fn get_project_harness(&self, _: &str) -> DomainResult<Option<ProjectHarnessRecord>> {
            Ok(self.0.lock().unwrap().clone())
        }
        fn save_project_harness(&self, record: &ProjectHarnessRecord) -> DomainResult<()> {
            *self.0.lock().unwrap() = Some(record.clone());
            Ok(())
        }
        fn delete_project_harness(&self, _: &str) -> DomainResult<()> {
            *self.0.lock().unwrap() = None;
            Ok(())
        }
    }

    #[test]
    fn status_detects_unmanaged_project_harness() {
        let root =
            std::env::temp_dir().join(format!("agentforge-project-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("AGENTS.md"), "# Agent").unwrap();
        fs::write(root.join("docs/harness.toml"), "id = 'x'").unwrap();
        let service = ProjectHarnessService::with_dirs(
            Arc::new(Projects {
                path: root.to_string_lossy().into_owned(),
            }),
            Arc::new(Templates { id: "t1".into() }),
            Arc::new(ProjectHarnesses(Mutex::new(None))),
            root.join("templates"),
        );
        let status = service.get_status("p1").unwrap();
        assert_eq!(status.state, "unmanaged_detected");
        fs::remove_dir_all(root).unwrap();
    }
}
