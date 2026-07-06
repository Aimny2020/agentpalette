use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs;
use crate::domain::ports::SkillRepository;
use crate::domain::skill::Skill;
use crate::domain::error::{DomainError, DomainResult};
use crate::infrastructure::markdown::parse_skill_markdown;

pub struct SkillService {
    repo: Arc<dyn SkillRepository>,
    skills_dir: PathBuf,
}

impl SkillService {
    pub fn new(repo: Arc<dyn SkillRepository>) -> Self {
        let home = dirs::home_dir().expect("Failed to locate home directory");
        let skills_dir = home.join(".agent-forge").join("skills");
        if !skills_dir.exists() {
            fs::create_dir_all(&skills_dir).expect("Failed to create skills directory");
        }
        Self { repo, skills_dir }
    }

    pub fn get_skills(&self) -> DomainResult<Vec<Skill>> {
        let mut list = Vec::new();
        if !self.skills_dir.exists() {
            return Ok(list);
        }
        for entry in fs::read_dir(&self.skills_dir).map_err(|e| DomainError::Database(e.to_string()))? {
            let entry = entry.map_err(|e| DomainError::Database(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                let skill_id = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                let skill_md_path = path.join("SKILL.md");
                if skill_md_path.exists() {
                    let content = fs::read_to_string(&skill_md_path).map_err(|e| DomainError::Database(e.to_string()))?;
                    if let Ok((metadata, html)) = parse_skill_markdown(&content) {
                        let (cat_id, notes) = match self.repo.get_user_meta(&skill_id)? {
                            Some(meta) => (meta.category_id, meta.user_notes),
                            None => (None, None),
                        };
                        list.push(Skill {
                            id: skill_id,
                            metadata,
                            html_content: html,
                            category_id: cat_id,
                            user_notes: notes,
                        });
                    }
                }
            }
        }
        Ok(list)
    }

    pub fn import_local_folder(&self, source_path: &str) -> DomainResult<String> {
        let src = Path::new(source_path);
        if !src.exists() || !src.is_dir() {
            return Err(DomainError::Database("Source directory does not exist".into()));
        }
        let md_path = src.join("SKILL.md");
        if !md_path.exists() {
            return Err(DomainError::Database("SKILL.md not found in source directory".into()));
        }
        let id = src.file_name().and_then(|s| s.to_str()).ok_or_else(|| DomainError::Database("Invalid folder name".into()))?;
        let dest = self.skills_dir.join(id);
        if dest.exists() {
            fs::remove_dir_all(&dest).map_err(|e| DomainError::Database(e.to_string()))?;
        }
        self.copy_dir_all(src, &dest).map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(id.to_string())
    }

    pub fn import_git_url(&self, url: &str) -> DomainResult<String> {
        let repo_name = url.split('/')
            .last()
            .and_then(|s| s.strip_suffix(".git").or(Some(s)))
            .ok_or_else(|| DomainError::Database("Invalid Git URL".into()))?;
        let dest = self.skills_dir.join(repo_name);
        if dest.exists() {
            fs::remove_dir_all(&dest).map_err(|e| DomainError::Database(e.to_string()))?;
        }
        // Run command git clone
        let status = std::process::Command::new("git")
            .args(["clone", url, dest.to_str().unwrap()])
            .status()
            .map_err(|e| DomainError::Database(format!("Git clone execution error: {}", e)))?;
        
        if !status.success() {
            return Err(DomainError::Database("git clone command exited with error".into()));
        }
        let md_path = dest.join("SKILL.md");
        if !md_path.exists() {
            return Err(DomainError::Database("Imported git repository does not contain a SKILL.md".into()));
        }
        Ok(repo_name.to_string())
    }

    pub fn delete_skill(&self, skill_id: &str) -> DomainResult<()> {
        let path = self.skills_dir.join(skill_id);
        if path.exists() && path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| DomainError::Database(e.to_string()))?;
        }
        self.repo.delete_user_meta(skill_id)?;
        Ok(())
    }

    pub fn toggle_project_skill(&self, project_id: &str, skill_id: &str, enabled: bool) -> DomainResult<()> {
        // 1. Save state to database
        self.repo.save_project_skill(project_id, skill_id, enabled)?;

        // 2. Fetch project directory path
        let project_path_str = match self.repo.get_project_path(project_id)? {
            Some(path) => path,
            None => return Err(DomainError::Database(format!("Project with ID {} not found", project_id))),
        };
        let project_path = Path::new(&project_path_str);
        
        // 3. Define target .agentforge/skills/<skill_id> folder
        let dest_dir = project_path.join(".agentforge").join("skills").join(skill_id);

        if enabled {
            // Copy from global library (~/.agent-forge/skills/<skill_id>) to project path
            let src_dir = self.skills_dir.join(skill_id);
            if src_dir.exists() {
                self.copy_dir_all(&src_dir, &dest_dir)
                    .map_err(|e| DomainError::Database(format!("Failed to copy skill: {}", e)))?;
            } else {
                return Err(DomainError::Database(format!("Global skill directory not found: {:?}", src_dir)));
            }
        } else {
            // Delete folder under project path
            if dest_dir.exists() {
                fs::remove_dir_all(&dest_dir)
                    .map_err(|e| DomainError::Database(format!("Failed to remove skill directory: {}", e)))?;
            }
        }

        Ok(())
    }

    fn copy_dir_all(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_dir() {
                self.copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}
