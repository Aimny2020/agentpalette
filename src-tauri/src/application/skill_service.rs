use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use crate::application::skill_scanner::scan_skill_root;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::ports::SkillRepository;
use crate::domain::skill::{
    ImportInspection, Skill, SkillKind, SkillMember, SkillPackageRecord, SkillSourceInfo,
    SkillUpdate, SourceKind, UpdateStatus,
};

pub struct SkillService {
    repo: Arc<dyn SkillRepository>,
    skills_dir: PathBuf,
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;

    use crate::domain::error::DomainResult;
    use crate::domain::ports::SkillRepository;
    use crate::domain::project::Project;
    use crate::domain::skill::{Category, UserSkillMeta};

    use super::{
        git_worktree_is_dirty, normalize_git_url, select_latest_stable_tag, trust_matches,
        SkillService,
    };

    #[derive(Default)]
    struct EmptyRepository {
        projects_using: Vec<String>,
    }

    impl SkillRepository for EmptyRepository {
        fn get_projects(&self) -> DomainResult<Vec<Project>> {
            Ok(Vec::new())
        }
        fn get_project_path(&self, _id: &str) -> DomainResult<Option<String>> {
            Ok(None)
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
        fn get_skill_package(
            &self,
            _skill_id: &str,
        ) -> DomainResult<Option<crate::domain::skill::SkillPackageRecord>> {
            Ok(None)
        }
        fn save_skill_package(
            &self,
            _record: &crate::domain::skill::SkillPackageRecord,
        ) -> DomainResult<()> {
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
            Ok(self.projects_using.clone())
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
            id: &str,
            name: &str,
            created_at: &str,
        ) -> DomainResult<Category> {
            Ok(Category {
                id: id.into(),
                name: name.into(),
                created_at: created_at.into(),
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

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("agentforge-service-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn skill(&self, relative: &str, name: &str) {
            let path = self.0.join(relative);
            fs::create_dir_all(&path).unwrap();
            fs::write(
                path.join("SKILL.md"),
                format!("---\nname: {name}\ndescription: {name} description\n---\n# {name}"),
            )
            .unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn get_skills_returns_nested_pack_as_one_catalog_entry() {
        let fixture = Fixture::new();
        fixture.skill("taste-skill/skills/alpha", "Alpha");
        fixture.skill("taste-skill/skills/beta", "Beta");
        let service =
            SkillService::with_skills_dir(Arc::new(EmptyRepository::default()), fixture.0.clone());

        let skills = service.get_skills().unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, "taste-skill");
        assert_eq!(skills[0].kind.as_str(), "pack");
        assert_eq!(skills[0].members.len(), 2);
    }

    #[test]
    fn imports_a_local_skill_pack_without_a_root_skill_file() {
        let fixture = Fixture::new();
        fixture.skill("source-pack/skills/alpha", "Alpha");
        fixture.skill("source-pack/skills/beta", "Beta");
        let library = fixture.0.join("library");
        fs::create_dir_all(&library).unwrap();
        let service =
            SkillService::with_skills_dir(Arc::new(EmptyRepository::default()), library.clone());

        let imported = service
            .import_local_folder(fixture.0.join("source-pack").to_str().unwrap())
            .unwrap();

        assert_eq!(imported, "source-pack");
        assert!(library.join("source-pack/skills/alpha/SKILL.md").exists());
    }

    #[test]
    fn normalizes_common_git_url_spellings_for_deduplication() {
        assert_eq!(
            normalize_git_url("https://github.com/obra/superpowers.git").unwrap(),
            "github.com/obra/superpowers"
        );
        assert_eq!(
            normalize_git_url("git@github.com:obra/superpowers.git").unwrap(),
            "github.com/obra/superpowers"
        );
    }

    #[test]
    fn copy_dir_all_excludes_git_and_github() {
        let fixture = Fixture::new();
        let src = fixture.0.join("src");
        let dst = fixture.0.join("dst");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("SKILL.md"), "hello").unwrap();

        let git_dir = src.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "git-config").unwrap();

        let github_dir = src.join(".github");
        fs::create_dir_all(&github_dir).unwrap();
        fs::write(github_dir.join("workflows.yml"), "workflow").unwrap();

        let nested_dir = src.join("nested");
        fs::create_dir_all(&nested_dir).unwrap();
        fs::write(nested_dir.join("file.txt"), "content").unwrap();
        let nested_git = nested_dir.join(".git");
        fs::create_dir_all(&nested_git).unwrap();
        fs::write(nested_git.join("nested-config"), "nested-git").unwrap();

        let service =
            SkillService::with_skills_dir(Arc::new(EmptyRepository::default()), fixture.0.clone());
        service.copy_dir_all(&src, &dst).unwrap();

        assert!(dst.join("SKILL.md").exists());
        assert!(dst.join("nested/file.txt").exists());
        assert!(!dst.join(".git").exists());
        assert!(!dst.join(".github").exists());
        assert!(!dst.join("nested/.git").exists());
    }

    #[test]
    fn refuses_to_delete_a_pack_used_by_projects() {
        let fixture = Fixture::new();
        fixture.skill("shared/SKILL", "Shared");
        let repository = EmptyRepository {
            projects_using: vec!["project-1".into()],
        };
        let service = SkillService::with_skills_dir(Arc::new(repository), fixture.0.clone());

        let error = service.delete_skill("shared").unwrap_err().to_string();

        assert!(error.contains("project-1"));
        assert!(fixture.0.join("shared").exists());
    }

    #[test]
    fn selects_highest_stable_semantic_tag() {
        let refs = "aaa refs/tags/v5.9.0\nbbb refs/tags/v6.1.1\nccc refs/tags/v6.2.0-beta.1\n";

        assert_eq!(select_latest_stable_tag(refs), Some("v6.1.1".into()));
    }

    #[test]
    fn detects_dirty_git_worktrees() {
        let fixture = Fixture::new();
        fixture.skill("repo", "Repo");
        let repo = fixture.0.join("repo");
        run_git(&repo, &["init"]);
        run_git(&repo, &["config", "user.email", "agentforge@example.test"]);
        run_git(&repo, &["config", "user.name", "AgentForge Test"]);
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "initial"]);
        assert!(!git_worktree_is_dirty(&repo).unwrap());

        fs::write(repo.join("SKILL.md"), "changed").unwrap();

        assert!(git_worktree_is_dirty(&repo).unwrap());
    }

    #[test]
    fn dirty_content_invalidates_commit_trust() {
        assert!(trust_matches(Some("abc"), Some("abc"), false));
        assert!(!trust_matches(Some("abc"), Some("abc"), true));
        assert!(!trust_matches(Some("abc"), Some("def"), false));
    }

    fn run_git(path: &std::path::Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[test]
    fn generates_correct_git_skill_ids() {
        use super::generate_git_skill_id;
        assert_eq!(generate_git_skill_id("github.com/mattpocock/skills").unwrap(), "mattpocock-skills");
        assert_eq!(generate_git_skill_id("github.com/axtonliu/axton-obsidian-visual-skills").unwrap(), "axtonliu-axton-obsidian-visual-skills");
        assert_eq!(generate_git_skill_id("github.com/org/Name-With_special-Chars!!").unwrap(), "org-name-with-special-chars");
        assert_eq!(generate_git_skill_id("github.com/org/---consecutive---dashes---").unwrap(), "org-consecutive-dashes");
    }

    #[test]
    fn stable_hash_generation() {
        use super::stable_hash;
        let h1 = stable_hash("github.com/mattpocock/skills");
        let h2 = stable_hash("github.com/mattpocock/skills");
        let h3 = stable_hash("github.com/another/skills");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 6);
    }

    #[test]
    fn integrates_git_skill_migration_on_service_startup() {
        use crate::infrastructure::database::SqliteDatabase;
        use crate::domain::skill::SkillPackageRecord;
        
        let db = Arc::new(SqliteDatabase::open_in_memory().unwrap());
        
        // Save old record
        let record = SkillPackageRecord {
            skill_id: "skills".into(),
            source_kind: crate::domain::skill::SourceKind::Git,
            source_url: Some("https://github.com/mattpocock/skills.git".into()),
            normalized_source: Some("github.com/mattpocock/skills".into()),
            tracked_ref: Some("v1.0.1".into()),
            installed_commit: Some("c123".into()),
            trusted_commit: None,
            last_checked_at: Some("2026-07-09T00:00:00Z".into()),
        };
        db.save_skill_package(&record).unwrap();
        
        let fixture = Fixture::new();
        // Create old skills folder
        let old_folder = fixture.0.join("skills");
        fs::create_dir_all(&old_folder).unwrap();
        fs::write(old_folder.join("SKILL.md"), "---\nname: mattpocock-skills\ndescription: desc\n---\n").unwrap();
        
        // Instantiate service to run migration on startup
        let _service = SkillService::with_skills_dir(db.clone(), fixture.0.clone());
        
        // Verify folder is renamed
        assert!(!old_folder.exists());
        assert!(fixture.0.join("mattpocock-skills").exists());
        
        // Verify DB record is migrated
        let new_record = db.get_skill_package("mattpocock-skills").unwrap().unwrap();
        assert_eq!(new_record.normalized_source.as_deref(), Some("github.com/mattpocock/skills"));
        assert!(db.get_skill_package("skills").unwrap().is_none());
    }

    #[test]
    fn manages_project_skills_json_dynamically() {
        let fixture = Fixture::new();
        // Create project and fake skill pack
        let db = Arc::new(crate::infrastructure::database::SqliteDatabase::open_in_memory().unwrap());
        let project = crate::domain::project::Project {
            id: "p1".into(),
            name: "Project 1".into(),
            path: fixture.0.to_str().unwrap().into(),
            created_at: "2026-07-09T00:00:00Z".into(),
        };
        db.create_project(&project).unwrap();

        // Put a fake pack in global skills dir
        let pack_dir = fixture.0.join("obra-superpowers");
        fs::create_dir_all(pack_dir.join("skills").join("brainstorming")).unwrap();
        fs::write(pack_dir.join("skills").join("brainstorming").join("SKILL.md"), "---\nname: Brainstorming\ndescription: desc\n---\n").unwrap();
        // Make sure it scans as pack (needs > 1 definition)
        fs::create_dir_all(pack_dir.join("skills").join("executing")).unwrap();
        fs::write(pack_dir.join("skills").join("executing").join("SKILL.md"), "---\nname: Executing\ndescription: desc\n---\n").unwrap();

        // Save provenance
        let record = crate::domain::skill::SkillPackageRecord {
            skill_id: "obra-superpowers".into(),
            source_kind: crate::domain::skill::SourceKind::Git,
            source_url: Some("https://github.com/obra/superpowers.git".into()),
            normalized_source: Some("github.com/obra/superpowers".into()),
            tracked_ref: Some("main".into()),
            installed_commit: Some("c1".into()),
            trusted_commit: Some("c1".into()),
            last_checked_at: Some("2026-07-09T00:00:00Z".into()),
        };
        db.save_skill_package(&record).unwrap();

        let service = SkillService::with_skills_dir(db.clone(), fixture.0.clone());

        // Toggle sub-skill brainstorming
        service.toggle_project_skill("p1", "obra-superpowers::skills/brainstorming", true).unwrap();

        // Verify skills.json is created
        let json_path = fixture.0.join(".agents").join("skills.json");
        assert!(json_path.exists());
        let content = fs::read_to_string(&json_path).unwrap();
        assert!(content.contains(".agents/skills/obra-superpowers/skills/brainstorming"));

        // Toggle executing plan as well
        service.toggle_project_skill("p1", "obra-superpowers::skills/executing", true).unwrap();
        let content2 = fs::read_to_string(&json_path).unwrap();
        assert!(content2.contains(".agents/skills/obra-superpowers/skills/brainstorming"));
        assert!(content2.contains(".agents/skills/obra-superpowers/skills/executing"));

        // Disable brainstorming
        service.toggle_project_skill("p1", "obra-superpowers::skills/brainstorming", false).unwrap();
        let content3 = fs::read_to_string(&json_path).unwrap();
        assert!(!content3.contains(".agents/skills/obra-superpowers/skills/brainstorming"));
        assert!(content3.contains(".agents/skills/obra-superpowers/skills/executing"));

        // Disable executing
        service.toggle_project_skill("p1", "obra-superpowers::skills/executing", false).unwrap();
        // Since no more custom skills are enabled, skills.json should be cleaned up and deleted
        assert!(!json_path.exists());
    }
}

impl SkillService {
    pub fn new(repo: Arc<dyn SkillRepository>) -> Self {
        let home = dirs::home_dir().expect("Failed to locate home directory");
        let skills_dir = home.join(".agent-forge").join("skills");
        if !skills_dir.exists() {
            fs::create_dir_all(&skills_dir).expect("Failed to create skills directory");
        }
        let service = Self { repo, skills_dir };
        if let Err(e) = service.migrate_old_git_skills() {
            eprintln!("Failed to migrate old Git skill IDs on startup: {:?}", e);
        }
        service
    }

    #[cfg(test)]
    fn with_skills_dir(repo: Arc<dyn SkillRepository>, skills_dir: PathBuf) -> Self {
        let service = Self { repo, skills_dir };
        if let Err(e) = service.migrate_old_git_skills() {
            eprintln!("Failed to migrate old Git skill IDs on startup: {:?}", e);
        }
        service
    }

    pub fn get_skills(&self) -> DomainResult<Vec<Skill>> {
        let mut list = Vec::new();
        if !self.skills_dir.exists() {
            return Ok(list);
        }
        let mut entries = fs::read_dir(&self.skills_dir)
            .map_err(|e| DomainError::Database(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| DomainError::Database(error.to_string()))?;
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }
            let skill_id = entry.file_name().to_string_lossy().into_owned();
            if let Ok(mut discovered) = scan_skill_root(&skill_id, &path) {
                let (category_id, user_notes) = match self.repo.get_user_meta(&skill_id)? {
                    Some(meta) => (meta.category_id, meta.user_notes),
                    None => (None, None),
                };
                let custom_description = self.repo.get_custom_description(&skill_id)?;
                let mut members = Vec::new();
                for member in discovered.members {
                    let m_desc = self.repo.get_custom_description(&member.id)?;
                    members.push(SkillMember {
                        id: member.id,
                        relative_path: member.relative_path,
                        metadata: member.metadata,
                        html_content: member.html_content,
                        custom_description: m_desc,
                    });
                }

                let filesystem_source = git_source(&path).unwrap_or_else(SkillSourceInfo::local);
                let mut record = self
                    .repo
                    .get_skill_package(&skill_id)?
                    .unwrap_or_else(|| package_record(&skill_id, &path, &filesystem_source));
                if filesystem_source.kind == SourceKind::Git {
                    record.source_kind = SourceKind::Git;
                    record.source_url = filesystem_source.url.clone();
                    record.normalized_source = filesystem_source
                        .url
                        .as_deref()
                        .and_then(|url| normalize_git_url(url).ok());
                    if let Some(normalized) = record.normalized_source.as_deref() {
                        if let Some(existing) = self.repo.find_skill_by_source(normalized)? {
                            if existing != skill_id {
                                discovered
                                    .warnings
                                    .push(format!("Git 来源与已安装的 {existing} 重复"));
                                record.normalized_source = None;
                            }
                        }
                    }
                    record.tracked_ref = filesystem_source.tracked_ref.clone();
                    record.installed_commit = filesystem_source.installed_commit.clone();
                } else {
                    record.installed_commit = Some(local_revision(&path));
                }
                let dirty = filesystem_source.kind == SourceKind::Git
                    && git_worktree_is_dirty(&path).unwrap_or(true);
                let trusted = trust_matches(
                    record.trusted_commit.as_deref(),
                    record.installed_commit.as_deref(),
                    dirty,
                );
                self.repo.save_skill_package(&record)?;
                let source = SkillSourceInfo {
                    kind: record.source_kind,
                    url: record.source_url.clone(),
                    tracked_ref: record.tracked_ref.clone(),
                    installed_commit: record.installed_commit.clone(),
                };
                let update_status = if source.kind == SourceKind::Git {
                    if dirty {
                        UpdateStatus::Dirty
                    } else {
                        UpdateStatus::Unknown
                    }
                } else {
                    UpdateStatus::NotApplicable
                };
                list.push(Skill {
                    id: discovered.id,
                    kind: discovered.kind,
                    metadata: discovered.metadata,
                    html_content: discovered.html_content,
                    members,
                    category_id,
                    user_notes,
                    source,
                    update_status,
                    available_commit: None,
                    has_executable_content: discovered.has_executable_content,
                    trusted,
                    warnings: discovered.warnings,
                    custom_description,
                });
            }
        }
        Ok(list)
    }

    pub fn import_local_folder(&self, source_path: &str) -> DomainResult<String> {
        let src = Path::new(source_path);
        if !src.exists() || !src.is_dir() {
            return Err(DomainError::Database(
                "Source directory does not exist".into(),
            ));
        }
        let id = src
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| DomainError::Database("Invalid folder name".into()))?;
        scan_skill_root(id, src)?;
        let dest = self.skills_dir.join(id);
        if dest.exists() {
            return Err(DomainError::Database(format!(
                "Skill {id} is already installed"
            )));
        }
        self.copy_dir_all(src, &dest)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let source = SkillSourceInfo::local();
        self.repo
            .save_skill_package(&package_record(id, &dest, &source))?;
        Ok(id.to_string())
    }

    pub fn inspect_import(
        &self,
        source: &str,
        import_type: &str,
    ) -> DomainResult<ImportInspection> {
        if import_type != "git" {
            let path = Path::new(source);
            let id = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| DomainError::Database("Invalid folder name".into()))?;
            let discovered = scan_skill_root(id, path)?;
            return Ok(inspection_from_discovered(discovered, None, None, id.to_string(), None));
        }

        let normalized = normalize_git_url(source)?;
        if let Some(existing) = self.repo.find_skill_by_source(&normalized)? {
            return Ok(ImportInspection {
                name: existing.clone(),
                kind: SkillKind::Pack,
                member_count: 0,
                has_executable_content: false,
                warnings: Vec::new(),
                recommended_ref: None,
                duplicate_skill_id: Some(existing.clone()),
                install_id: existing,
                normalized_source: Some(normalized),
            });
        }
        let install_id = self.resolve_git_skill_id(&normalized)?;
        let recommended_ref = latest_stable_tag(source);
        let staging = self
            .skills_dir
            .join(format!(".{}-inspect-{}", install_id, uuid::Uuid::new_v4()));
        let status = clone_repository(source, recommended_ref.as_deref(), &staging)?;
        if !status.success() {
            let _ = fs::remove_dir_all(&staging);
            return Err(DomainError::Database(
                "git clone command exited with error".into(),
            ));
        }
        let result = scan_skill_root(&install_id, &staging)
            .map(|discovered| inspection_from_discovered(discovered, recommended_ref, None, install_id, Some(normalized)));
        let _ = fs::remove_dir_all(&staging);
        result
    }

    pub fn import_git_url(&self, url: &str) -> DomainResult<String> {
        let normalized = normalize_git_url(url)?;
        if let Some(existing) = self.repo.find_skill_by_source(&normalized)? {
            return Err(DomainError::Database(format!(
                "Git source is already installed as {existing}"
            )));
        }
        let install_id = self.resolve_git_skill_id(&normalized)?;
        let dest = self.skills_dir.join(&install_id);
        if dest.exists() {
            return Err(DomainError::Database(format!(
                "Skill {install_id} is already installed"
            )));
        }
        let staging = self
            .skills_dir
            .join(format!(".{}-import-{}", install_id, uuid::Uuid::new_v4()));
        let stable_tag = latest_stable_tag(url);
        let status = clone_repository(url, stable_tag.as_deref(), &staging)?;

        if !status.success() {
            let _ = fs::remove_dir_all(&staging);
            return Err(DomainError::Database(
                "git clone command exited with error".into(),
            ));
        }
        if let Err(error) = scan_skill_root(&install_id, &staging) {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
        fs::rename(&staging, &dest).map_err(|error| DomainError::Database(error.to_string()))?;
        let source = git_source(&dest)
            .ok_or_else(|| DomainError::Database("Git metadata is unavailable".into()))?;
        let mut record = package_record(&install_id, &dest, &source);
        record.source_url = Some(url.to_string());
        record.normalized_source = Some(normalized);
        self.repo.save_skill_package(&record)?;
        Ok(install_id)
    }

    pub fn check_skill_updates(&self) -> DomainResult<Vec<SkillUpdate>> {
        let mut updates = Vec::new();
        for skill in self.get_skills()? {
            if skill.source.kind != SourceKind::Git {
                continue;
            }
            let path = self.skills_dir.join(&skill.id);
            let installed_commit = skill.source.installed_commit.clone();
            let (status, available_commit) = if git_worktree_is_dirty(&path)? {
                (UpdateStatus::Dirty, None)
            } else if let (Some(url), Some(tracked_ref)) = (
                skill.source.url.as_deref(),
                skill.source.tracked_ref.as_deref(),
            ) {
                match resolve_remote_target(url, tracked_ref).map(|(_, commit)| commit) {
                    Some(remote) if Some(&remote) == installed_commit.as_ref() => {
                        (UpdateStatus::Current, Some(remote))
                    }
                    Some(remote) => (UpdateStatus::Available, Some(remote)),
                    None => (UpdateStatus::Unknown, None),
                }
            } else {
                (UpdateStatus::Unknown, None)
            };
            if let Some(mut record) = self.repo.get_skill_package(&skill.id)? {
                record.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
                self.repo.save_skill_package(&record)?;
            }
            updates.push(SkillUpdate {
                skill_id: skill.id,
                status,
                installed_commit,
                available_commit,
            });
        }
        Ok(updates)
    }

    pub fn update_skill(&self, skill_id: &str) -> DomainResult<SkillUpdate> {
        let path = self.skills_dir.join(skill_id);
        if git_worktree_is_dirty(&path)? {
            return Err(DomainError::Database(format!(
                "Skill Pack {skill_id} has local modifications"
            )));
        }
        let mut record = self
            .repo
            .get_skill_package(skill_id)?
            .ok_or_else(|| DomainError::Database("Skill Pack provenance is unavailable".into()))?;
        let url = record
            .source_url
            .clone()
            .ok_or_else(|| DomainError::Database("Git remote is unavailable".into()))?;
        let tracked_ref = record
            .tracked_ref
            .clone()
            .ok_or_else(|| DomainError::Database("Tracked Git ref is unavailable".into()))?;
        let (available_ref, available_commit) = resolve_remote_target(&url, &tracked_ref)
            .ok_or_else(|| DomainError::Database("Unable to resolve remote Git ref".into()))?;
        if record.installed_commit.as_deref() == Some(&available_commit) {
            return Ok(SkillUpdate {
                skill_id: skill_id.into(),
                status: UpdateStatus::Current,
                installed_commit: record.installed_commit,
                available_commit: Some(available_commit),
            });
        }

        let staging = self
            .skills_dir
            .join(format!(".{skill_id}-update-{}", uuid::Uuid::new_v4()));
        let backup = self
            .skills_dir
            .join(format!(".{skill_id}-backup-{}", uuid::Uuid::new_v4()));
        let status = clone_repository(&url, Some(&available_ref), &staging)?;
        if !status.success() {
            let _ = fs::remove_dir_all(&staging);
            return Err(DomainError::Database(
                "git clone command exited with error".into(),
            ));
        }
        scan_skill_root(skill_id, &staging)?;
        fs::rename(&path, &backup).map_err(|error| DomainError::Database(error.to_string()))?;
        if let Err(error) = fs::rename(&staging, &path) {
            let _ = fs::rename(&backup, &path);
            return Err(DomainError::Database(error.to_string()));
        }

        let source = git_source(&path)
            .ok_or_else(|| DomainError::Database("Updated Git metadata is unavailable".into()))?;
        record.installed_commit = source.installed_commit.clone();
        record.tracked_ref = source.tracked_ref.or(Some(available_ref));
        record.trusted_commit = None;
        record.last_checked_at = Some(chrono::Utc::now().to_rfc3339());
        self.repo.save_skill_package(&record)?;

        for project_id in self.repo.get_projects_using_skill(skill_id)? {
            if let Some(project_path) = self.repo.get_project_path(&project_id)? {
                let destination = Path::new(&project_path)
                    .join(".agents")
                    .join("skills")
                    .join(skill_id);
                let modified = destination.exists()
                    && destination.join(".git").exists()
                    && git_worktree_is_dirty(&destination)?;
                if !modified {
                    if destination.exists() {
                        fs::remove_dir_all(&destination)
                            .map_err(|error| DomainError::Database(error.to_string()))?;
                    }
                    self.copy_dir_all(&path, &destination)
                        .map_err(|error| DomainError::Database(error.to_string()))?;
                    self.repo.save_project_skill_state(
                        &project_id,
                        skill_id,
                        record.installed_commit.as_deref(),
                        "current",
                    )?;
                } else {
                    self.repo
                        .save_project_skill_state(&project_id, skill_id, None, "modified")?;
                }
            }
        }
        let _ = fs::remove_dir_all(&backup);

        Ok(SkillUpdate {
            skill_id: skill_id.into(),
            status: UpdateStatus::Current,
            installed_commit: record.installed_commit,
            available_commit: Some(available_commit),
        })
    }

    pub fn trust_skill(&self, skill_id: &str) -> DomainResult<()> {
        let path = self.skills_dir.join(skill_id);
        let mut record = self
            .repo
            .get_skill_package(skill_id)?
            .unwrap_or_else(|| package_record(skill_id, &path, &SkillSourceInfo::local()));
        record.trusted_commit = record.installed_commit.clone();
        self.repo.save_skill_package(&record)
    }

    pub fn delete_skill_everywhere(&self, skill_id: &str) -> DomainResult<()> {
        for project_id in self.repo.get_projects_using_skill(skill_id)? {
            self.toggle_project_skill(&project_id, skill_id, false)?;
        }
        self.delete_skill(skill_id)
    }

    pub fn delete_skill(&self, skill_id: &str) -> DomainResult<()> {
        let projects = self.repo.get_projects_using_skill(skill_id)?;
        if !projects.is_empty() {
            let all_projects = self.repo.get_projects()?;
            let mut formatted_projects = Vec::new();
            for pid in projects {
                if let Some(proj) = all_projects.iter().find(|p| p.id == pid) {
                    formatted_projects.push(format!("{} ({})", proj.name, proj.path));
                } else {
                    formatted_projects.push(pid);
                }
            }
            return Err(DomainError::Database(format!(
                "Skill Pack is enabled in projects: {}",
                formatted_projects.join(", ")
            )));
        }
        let path = self.skills_dir.join(skill_id);
        if path.exists() && path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| DomainError::Database(e.to_string()))?;
        }
        self.repo.delete_user_meta(skill_id)?;
        Ok(())
    }

    pub fn toggle_project_skill(
        &self,
        project_id: &str,
        skill_id: &str,
        enabled: bool,
    ) -> DomainResult<()> {
        let project_path_str = match self.repo.get_project_path(project_id)? {
            Some(path) => path,
            None => {
                return Err(DomainError::Database(format!(
                    "Project with ID {} not found",
                    project_id
                )))
            }
        };
        let project_path = Path::new(&project_path_str);

        if skill_id.contains("::") {
            // It is a sub-skill
            let parts: Vec<&str> = skill_id.split("::").collect();
            let pack_id = parts[0];
            let sub_path = parts[1];

            let pack_dest_dir = project_path.join(".agents").join("skills").join(pack_id);
            let pack_src_dir = self.skills_dir.join(pack_id);

            if enabled {
                if !pack_src_dir.exists() {
                    return Err(DomainError::Database(format!(
                        "Global skill directory not found: {:?}",
                        pack_src_dir
                    )));
                }

                // Verify trust for parent pack
                let discovered = scan_skill_root(pack_id, &pack_src_dir)?;
                if discovered.has_executable_content {
                    let record = self.repo.get_skill_package(pack_id)?;
                    let trusted = record.as_ref().is_some_and(|record| {
                        record.trusted_commit.is_some()
                            && record.trusted_commit == record.installed_commit
                    });
                    if !trusted {
                        return Err(DomainError::Database(format!(
                            "Skill Pack {pack_id} contains executable content and is not trusted"
                        )));
                    }
                }

                // Save sub-skill to database
                self.repo.save_project_skill(project_id, skill_id, true)?;
                // Ensure parent package row is also saved as enabled
                self.repo.save_project_skill(project_id, pack_id, true)?;

                // Copy package folder if not exists
                if !pack_dest_dir.exists() {
                    self.copy_dir_all(&pack_src_dir, &pack_dest_dir)
                        .map_err(|e| {
                            DomainError::Database(format!("Failed to copy skill: {}", e))
                        })?;

                    let commit = self
                        .repo
                        .get_skill_package(pack_id)?
                        .and_then(|record| record.installed_commit);
                    self.repo.save_project_skill_state(
                        project_id,
                        pack_id,
                        commit.as_deref(),
                        "current",
                    )?;
                }

                // Sync all members (will activate this one, and keep others disabled if they are disabled)
                self.sync_package_members(project_id, &discovered, &pack_dest_dir)?;
            } else {
                // Disable sub-skill
                self.repo.save_project_skill(project_id, skill_id, false)?;

                let pack_src_dir = self.skills_dir.join(pack_id);
                if pack_src_dir.exists() {
                    let _ = scan_skill_root(pack_id, &pack_src_dir)?;

                    // Rename its SKILL.md to SKILL.md.disabled
                    let member_skill_md = pack_dest_dir.join(sub_path).join("SKILL.md");
                    let member_skill_md_disabled =
                        pack_dest_dir.join(sub_path).join("SKILL.md.disabled");
                    if member_skill_md.exists() {
                        fs::rename(&member_skill_md, &member_skill_md_disabled)
                            .map_err(|e| DomainError::Database(e.to_string()))?;
                    }

                    // Check if other sub-skills of the same package are still enabled
                    let enabled_skills = self.repo.get_project_skills(project_id)?;
                    let has_other_enabled = enabled_skills
                        .iter()
                        .any(|id| id.starts_with(&format!("{pack_id}::")));

                    if !has_other_enabled {
                        // No other sub-skills are enabled, clean up the package folder and parent database row
                        if pack_dest_dir.exists() {
                            fs::remove_dir_all(&pack_dest_dir)
                                .map_err(|e| DomainError::Database(e.to_string()))?;
                        }
                        self.repo.save_project_skill(project_id, pack_id, false)?;
                    }
                }
            }
        } else {
            // It is a standalone skill or the entire package toggled
            let dest_dir = project_path.join(".agents").join("skills").join(skill_id);
            let src_dir = self.skills_dir.join(skill_id);

            if src_dir.exists() {
                let discovered = scan_skill_root(skill_id, &src_dir)?;
                if discovered.kind == SkillKind::Pack {
                    // Package toggled
                    if enabled {
                        // Enable package
                        if discovered.has_executable_content {
                            let record = self.repo.get_skill_package(skill_id)?;
                            let trusted = record.as_ref().is_some_and(|record| {
                                record.trusted_commit.is_some()
                                    && record.trusted_commit == record.installed_commit
                            });
                            if !trusted {
                                return Err(DomainError::Database(format!(
                                    "Skill Pack {skill_id} contains executable content and is not trusted"
                                )));
                            }
                        }

                        // Enable package row and all member rows in DB
                        self.repo.save_project_skill(project_id, skill_id, true)?;
                        for member in &discovered.members {
                            self.repo.save_project_skill(project_id, &member.id, true)?;
                        }

                        // Copy entire folder
                        if dest_dir.exists() {
                            fs::remove_dir_all(&dest_dir)
                                .map_err(|e| DomainError::Database(e.to_string()))?;
                        }
                        self.copy_dir_all(&src_dir, &dest_dir).map_err(|e| {
                            DomainError::Database(format!("Failed to copy skill: {}", e))
                        })?;

                        // Activate all SKILL.md files (by syncing)
                        self.sync_package_members(project_id, &discovered, &dest_dir)?;

                        let commit = self
                            .repo
                            .get_skill_package(skill_id)?
                            .and_then(|record| record.installed_commit);
                        self.repo.save_project_skill_state(
                            project_id,
                            skill_id,
                            commit.as_deref(),
                            "current",
                        )?;
                    } else {
                        // Disable package and all members
                        self.repo.save_project_skill(project_id, skill_id, false)?;
                        for member in &discovered.members {
                            self.repo
                                .save_project_skill(project_id, &member.id, false)?;
                        }
                        if dest_dir.exists() {
                            fs::remove_dir_all(&dest_dir)
                                .map_err(|e| DomainError::Database(e.to_string()))?;
                        }
                    }
                } else {
                    // Standalone skill toggled
                    if enabled {
                        if discovered.has_executable_content {
                            let record = self.repo.get_skill_package(skill_id)?;
                            let trusted = record.as_ref().is_some_and(|record| {
                                record.trusted_commit.is_some()
                                    && record.trusted_commit == record.installed_commit
                            });
                            if !trusted {
                                return Err(DomainError::Database(format!(
                                    "Skill Pack {skill_id} contains executable content and is not trusted"
                                )));
                            }
                        }
                        if dest_dir.exists() {
                            fs::remove_dir_all(&dest_dir)
                                .map_err(|e| DomainError::Database(e.to_string()))?;
                        }
                        self.copy_dir_all(&src_dir, &dest_dir).map_err(|e| {
                            DomainError::Database(format!("Failed to copy skill: {}", e))
                        })?;
                        self.repo.save_project_skill(project_id, skill_id, true)?;
                        let commit = self
                            .repo
                            .get_skill_package(skill_id)?
                            .and_then(|record| record.installed_commit);
                        self.repo.save_project_skill_state(
                            project_id,
                            skill_id,
                            commit.as_deref(),
                            "current",
                        )?;
                    } else {
                        if dest_dir.exists() {
                            fs::remove_dir_all(&dest_dir)
                                .map_err(|e| DomainError::Database(e.to_string()))?;
                        }
                        self.repo.save_project_skill(project_id, skill_id, false)?;
                    }
                }
            } else {
                return Err(DomainError::Database(format!(
                    "Global skill directory not found: {:?}",
                    src_dir
                )));
            }
        }
        
        self.update_project_skills_json(project_path, project_id)?;
        Ok(())
    }

    fn sync_package_members(
        &self,
        project_id: &str,
        discovered: &crate::application::skill_scanner::DiscoveredSkill,
        dest_dir: &Path,
    ) -> DomainResult<()> {
        let enabled_skills = self.repo.get_project_skills(project_id)?;
        for member in &discovered.members {
            let is_member_enabled = enabled_skills.contains(&member.id);
            let member_skill_md = dest_dir.join(&member.relative_path).join("SKILL.md");
            let member_skill_md_disabled = dest_dir
                .join(&member.relative_path)
                .join("SKILL.md.disabled");

            if is_member_enabled {
                if member_skill_md_disabled.exists() && !member_skill_md.exists() {
                    fs::rename(&member_skill_md_disabled, &member_skill_md)
                        .map_err(|e| DomainError::Database(e.to_string()))?;
                }
            } else {
                if member_skill_md.exists() {
                    fs::rename(&member_skill_md, &member_skill_md_disabled)
                        .map_err(|e| DomainError::Database(e.to_string()))?;
                }
            }
        }
        Ok(())
    }

    fn copy_dir_all(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if file_name_str == ".git" || file_name_str == ".github" {
                continue;
            }
            if ty.is_symlink() {
                continue;
            }
            if ty.is_dir() {
                self.copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}

pub fn normalize_git_url(url: &str) -> DomainResult<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let without_scheme = if let Some(value) = trimmed.strip_prefix("https://") {
        value
    } else if let Some(value) = trimmed.strip_prefix("http://") {
        value
    } else if let Some(value) = trimmed.strip_prefix("ssh://") {
        value.trim_start_matches("git@")
    } else if let Some(value) = trimmed.strip_prefix("git@") {
        value
    } else {
        trimmed
    };
    let normalized = without_scheme
        .replace(':', "/")
        .trim_end_matches(".git")
        .trim_end_matches('/')
        .to_ascii_lowercase();
    if normalized.split('/').count() < 3 || normalized.contains(char::is_whitespace) {
        return Err(DomainError::Database("Invalid Git URL".into()));
    }
    Ok(normalized)
}

fn git_source(path: &Path) -> Option<SkillSourceInfo> {
    if !path.join(".git").exists() {
        return None;
    }
    let url = git_output(path, &["remote", "get-url", "origin"]);
    let installed_commit = git_output(path, &["rev-parse", "HEAD"]);
    let tracked_ref = git_output(path, &["symbolic-ref", "--short", "HEAD"])
        .or_else(|| git_output(path, &["describe", "--tags", "--exact-match"]));
    Some(SkillSourceInfo {
        kind: SourceKind::Git,
        url,
        tracked_ref,
        installed_commit,
    })
}

fn inspection_from_discovered(
    discovered: crate::application::skill_scanner::DiscoveredSkill,
    recommended_ref: Option<String>,
    duplicate_skill_id: Option<String>,
    install_id: String,
    normalized_source: Option<String>,
) -> ImportInspection {
    let member_count = if discovered.kind == SkillKind::Pack {
        discovered.members.len()
    } else {
        1
    };
    ImportInspection {
        name: discovered.metadata.name,
        kind: discovered.kind,
        member_count,
        has_executable_content: discovered.has_executable_content,
        warnings: discovered.warnings,
        recommended_ref,
        duplicate_skill_id,
        install_id,
        normalized_source,
    }
}

fn package_record(id: &str, path: &Path, source: &SkillSourceInfo) -> SkillPackageRecord {
    SkillPackageRecord {
        skill_id: id.to_string(),
        source_kind: source.kind,
        source_url: source.url.clone(),
        normalized_source: source
            .url
            .as_deref()
            .and_then(|url| normalize_git_url(url).ok()),
        tracked_ref: source.tracked_ref.clone(),
        installed_commit: source
            .installed_commit
            .clone()
            .or_else(|| Some(local_revision(path))),
        trusted_commit: None,
        last_checked_at: None,
    }
}

fn local_revision(root: &Path) -> String {
    let mut paths = Vec::new();
    collect_revision_paths(root, &mut paths);
    paths.sort();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for path in paths {
        path.strip_prefix(root).unwrap_or(&path).hash(&mut hasher);
        if let Ok(metadata) = fs::metadata(&path) {
            metadata.len().hash(&mut hasher);
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    duration.as_nanos().hash(&mut hasher);
                }
            }
        }
    }
    format!("local-{:016x}", hasher.finish())
}

fn collect_revision_paths(directory: &Path, paths: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() || entry.file_name() == ".git" {
            continue;
        }
        if file_type.is_dir() {
            collect_revision_paths(&path, paths);
        } else if file_type.is_file() {
            paths.push(path);
        }
    }
}

fn git_output(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn clone_repository(
    url: &str,
    tracked_ref: Option<&str>,
    destination: &Path,
) -> DomainResult<std::process::ExitStatus> {
    let mut command = Command::new("git");
    command.arg("clone").arg("--depth").arg("1");
    if let Some(reference) = tracked_ref {
        command.arg("--branch").arg(reference);
    }
    command
        .arg(url)
        .arg(destination)
        .status()
        .map_err(|error| DomainError::Database(format!("Git clone execution error: {error}")))
}

fn latest_stable_tag(url: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["ls-remote", "--tags", "--refs", url])
        .output()
        .ok()?;
    output.status.success().then_some(())?;
    select_latest_stable_tag(&String::from_utf8(output.stdout).ok()?)
}

fn select_latest_stable_tag(refs: &str) -> Option<String> {
    refs.lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .filter_map(|reference| reference.strip_prefix("refs/tags/"))
        .filter_map(|tag| semantic_version(tag).map(|version| (version, tag.to_string())))
        .max_by_key(|(version, _)| *version)
        .map(|(_, tag)| tag)
}

fn semantic_version(tag: &str) -> Option<(u64, u64, u64)> {
    let value = tag.strip_prefix('v').unwrap_or(tag);
    if value.contains('-') || value.contains('+') {
        return None;
    }
    let mut parts = value.split('.');
    let version = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    parts.next().is_none().then_some(version)
}

fn git_worktree_is_dirty(path: &Path) -> DomainResult<bool> {
    if !path.join(".git").exists() {
        return Ok(false);
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["status", "--porcelain"])
        .output()
        .map_err(|error| DomainError::Database(error.to_string()))?;
    if !output.status.success() {
        return Err(DomainError::Database(
            "Unable to inspect Git worktree".into(),
        ));
    }
    Ok(!output.stdout.is_empty())
}

fn trust_matches(
    trusted_commit: Option<&str>,
    installed_commit: Option<&str>,
    dirty: bool,
) -> bool {
    !dirty && trusted_commit.is_some() && trusted_commit == installed_commit
}

fn remote_commit(url: &str, tracked_ref: &str) -> Option<String> {
    let reference = if tracked_ref.starts_with("refs/") {
        tracked_ref.to_string()
    } else if semantic_version(tracked_ref).is_some() {
        format!("refs/tags/{tracked_ref}")
    } else {
        format!("refs/heads/{tracked_ref}")
    };
    let mut command = Command::new("git");
    command.args(["ls-remote", url, &reference]);
    if reference.starts_with("refs/tags/") {
        command.arg(format!("{reference}^{{}}"));
    }
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .next_back()
        .map(ToString::to_string)
}

fn resolve_remote_target(url: &str, tracked_ref: &str) -> Option<(String, String)> {
    let target_ref = if semantic_version(tracked_ref).is_some() {
        latest_stable_tag(url).unwrap_or_else(|| tracked_ref.to_string())
    } else {
        tracked_ref.to_string()
    };
    let commit = remote_commit(url, &target_ref)?;
    Some((target_ref, commit))
}

fn generate_git_skill_id(normalized: &str) -> DomainResult<String> {
    let parts: Vec<&str> = normalized.split('/').collect();
    if parts.len() < 2 {
        return Err(DomainError::Database(format!(
            "Invalid normalized Git source: {}",
            normalized
        )));
    }
    let owner = parts[parts.len() - 2];
    let repo = parts[parts.len() - 1];
    
    let clean = |s: &str| {
        s.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
            .collect::<String>()
            .to_ascii_lowercase()
    };
    
    let owner_clean = clean(owner);
    let repo_clean = clean(repo);
    let combined = format!("{}-{}", owner_clean, repo_clean);
    
    let mut compressed = String::new();
    let mut last_was_dash = false;
    for c in combined.chars() {
        if c == '-' {
            if !last_was_dash {
                compressed.push(c);
                last_was_dash = true;
            }
        } else {
            compressed.push(c);
            last_was_dash = false;
        }
    }
    let trimmed = compressed.trim_matches('-').to_string();
    if trimmed.is_empty() {
        return Err(DomainError::Database(format!(
            "Generated Git skill ID is empty for source: {}",
            normalized
        )));
    }
    Ok(trimmed)
}

fn stable_hash(s: &str) -> String {
    let mut hash: u32 = 5381;
    for c in s.chars() {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(c as u32);
    }
    format!("{:06x}", hash & 0xFFFFFF)
}

impl SkillService {
    fn resolve_git_skill_id(&self, normalized: &str) -> DomainResult<String> {
        let base_id = generate_git_skill_id(normalized)?;
        
        let path = self.skills_dir.join(&base_id);
        if path.exists() || self.repo.get_skill_package(&base_id)?.is_some() {
            let existing = self.repo.get_skill_package(&base_id)?;
            let is_same_source = existing.as_ref()
                .and_then(|r| r.normalized_source.as_deref()) == Some(normalized);
            
            if is_same_source {
                Ok(base_id)
            } else {
                let suffix = stable_hash(normalized);
                Ok(format!("{}-{}", base_id, suffix))
            }
        } else {
            Ok(base_id)
        }
    }

    fn generate_new_skill_id_for_migration(&self, old_id: &str, normalized: &str) -> DomainResult<String> {
        let base_new_id = generate_git_skill_id(normalized)?;
        let mut final_new_id = base_new_id.clone();
        
        let dest_base = self.skills_dir.join(&final_new_id);
        if (dest_base.exists() || self.repo.get_skill_package(&final_new_id)?.is_some()) && final_new_id != old_id {
            let existing_rec = self.repo.get_skill_package(&final_new_id)?;
            let is_same_source = existing_rec.as_ref()
                .and_then(|r| r.normalized_source.as_deref()) == Some(normalized);
            
            if is_same_source {
                return Err(DomainError::Database(format!(
                    "Migration conflict: both {} and {} exist for normalized source {}",
                    old_id, final_new_id, normalized
                )));
            } else {
                let suffix = stable_hash(normalized);
                final_new_id = format!("{}-{}", base_new_id, suffix);
            }
        }
        
        if final_new_id != old_id {
            let final_dest = self.skills_dir.join(&final_new_id);
            if final_dest.exists() || self.repo.get_skill_package(&final_new_id)?.is_some() {
                return Err(DomainError::Database(format!(
                    "Migration conflict: target {} already exists",
                    final_new_id
                )));
            }
        }
        
        Ok(final_new_id)
    }

    fn migrate_single_skill(&self, old_id: &str, new_id: &str) -> DomainResult<()> {
        let old_path = self.skills_dir.join(old_id);
        let new_path = self.skills_dir.join(new_id);
        
        let renamed_fs = if old_path.exists() && old_path.is_dir() {
            fs::rename(&old_path, &new_path)
                .map_err(|e| DomainError::Database(format!("Failed to rename skill folder: {}", e)))?;
            true
        } else {
            false
        };
        
        let db_result = self.repo.migrate_git_skill_id(old_id, new_id);
        if db_result.is_err() {
            if renamed_fs {
                let _ = fs::rename(&new_path, &old_path);
            }
            return db_result;
        }
        
        Ok(())
    }

    pub fn migrate_old_git_skills(&self) -> DomainResult<()> {
        if !self.skills_dir.exists() {
            return Ok(());
        }
        
        let entries = fs::read_dir(&self.skills_dir)
            .map_err(|e| DomainError::Database(e.to_string()))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            
            let old_id = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => continue,
            };
            
            let record = match self.repo.get_skill_package(old_id)? {
                Some(r) => r,
                None => continue,
            };
            
            if record.source_kind != SourceKind::Git {
                continue;
            }
            
            let normalized = match record.normalized_source.as_deref() {
                Some(url) => url,
                None => continue,
            };
            
            let new_id = self.generate_new_skill_id_for_migration(old_id, normalized)?;
            if new_id != old_id {
                self.migrate_single_skill(old_id, &new_id)?;
            }
        }
        
        Ok(())
    }

    fn update_project_skills_json(&self, project_path: &Path, project_id: &str) -> DomainResult<()> {
        let enabled_skills = self.repo.get_project_skills(project_id)?;
        let mut custom_paths = Vec::new();
        for skill_id in enabled_skills {
            if skill_id.contains("::") {
                let parts: Vec<&str> = skill_id.split("::").collect();
                let pack_id = parts[0];
                let sub_path = parts[1];
                custom_paths.push(format!(".agents/skills/{}/{}", pack_id, sub_path));
            }
        }
        
        let skills_json_path = project_path.join(".agents").join("skills.json");
        
        #[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
        struct SkillsJsonEntry {
            path: String,
        }

        #[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
        struct SkillsJson {
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            entries: Vec<SkillsJsonEntry>,
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            inherits: Vec<SkillsJsonEntry>,
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            exclude: Vec<String>,
        }
        
        let mut config = if skills_json_path.exists() {
            let content = fs::read_to_string(&skills_json_path)
                .map_err(|e| DomainError::Database(format!("Failed to read skills.json: {}", e)))?;
            serde_json::from_str::<SkillsJson>(&content)
                .unwrap_or_default()
        } else {
            SkillsJson::default()
        };
        
        config.entries = custom_paths
            .into_iter()
            .map(|path| SkillsJsonEntry { path })
            .collect();
            
        if config.entries.is_empty() && config.inherits.is_empty() && config.exclude.is_empty() {
            if skills_json_path.exists() {
                let _ = fs::remove_file(&skills_json_path);
            }
        } else {
            let agents_dir = project_path.join(".agents");
            if !agents_dir.exists() {
                fs::create_dir_all(&agents_dir)
                    .map_err(|e| DomainError::Database(format!("Failed to create .agents directory: {}", e)))?;
            }
            let content = serde_json::to_string_pretty(&config)
                .map_err(|e| DomainError::Database(format!("Failed to serialize skills.json: {}", e)))?;
            fs::write(&skills_json_path, content)
                .map_err(|e| DomainError::Database(format!("Failed to write skills.json: {}", e)))?;
        }
        
        Ok(())
    }
}
