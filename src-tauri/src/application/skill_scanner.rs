use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::domain::error::{DomainError, DomainResult};
use crate::domain::skill::{SkillKind, SkillMember, SkillMetadata};
use crate::infrastructure::markdown::parse_skill_markdown;

const IGNORED_DIRECTORIES: &[&str] = &[".git", "node_modules", "target", "__pycache__", ".cache"];
const EXECUTABLE_EXTENSIONS: &[&str] = &[
    "sh", "bash", "zsh", "fish", "py", "js", "mjs", "cjs", "ts", "ps1", "bat", "cmd",
];

#[derive(Debug)]
pub struct DiscoveredSkill {
    pub id: String,
    pub kind: SkillKind,
    pub metadata: SkillMetadata,
    pub html_content: String,
    pub members: Vec<SkillMember>,
    pub source_path: PathBuf,
    pub has_executable_content: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
struct ParsedSkill {
    relative_directory: String,
    source_path: PathBuf,
    metadata: SkillMetadata,
    html_content: String,
}

#[derive(Deserialize)]
struct PluginManifest {
    name: String,
    description: Option<String>,
    version: Option<String>,
    author: Option<ManifestAuthor>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ManifestAuthor {
    Name(String),
    Object { name: String },
}

impl ManifestAuthor {
    fn into_name(self) -> String {
        match self {
            Self::Name(name) | Self::Object { name } => name,
        }
    }
}

pub fn scan_skill_root(id: &str, root: &Path) -> DomainResult<DiscoveredSkill> {
    let mut markdown_paths = Vec::new();
    walk(root, &mut markdown_paths)?;
    markdown_paths.sort();
    let definition_count = markdown_paths.len();

    let mut parsed = Vec::new();
    let mut warnings = Vec::new();
    for path in markdown_paths {
        let content =
            fs::read_to_string(&path).map_err(|error| DomainError::Database(error.to_string()))?;
        match parse_skill_markdown(&content) {
            Ok((metadata, html_content)) => {
                let directory = path.parent().unwrap_or(root).to_path_buf();
                let relative_directory = directory
                    .strip_prefix(root)
                    .unwrap_or(&directory)
                    .to_string_lossy()
                    .replace('\\', "/");
                parsed.push(ParsedSkill {
                    relative_directory,
                    source_path: directory,
                    metadata,
                    html_content,
                });
            }
            Err(error) => warnings.push(format!("{}: {}", path.display(), error)),
        }
    }

    if parsed.is_empty() {
        return Err(DomainError::Database(format!(
            "No valid SKILL.md found in {}",
            root.display()
        )));
    }

    let mut has_executable_content = false;
    for skill in &parsed {
        if directory_has_executable_content(&skill.source_path)? {
            has_executable_content = true;
            break;
        }
    }

    let is_pack = definition_count > 1;
    if !is_pack {
        let only = parsed.remove(0);
        return Ok(DiscoveredSkill {
            id: id.to_string(),
            kind: SkillKind::Standalone,
            metadata: only.metadata,
            html_content: only.html_content,
            members: Vec::new(),
            source_path: only.source_path,
            has_executable_content,
            warnings,
        });
    }

    let root_skill = parsed
        .iter()
        .find(|skill| skill.relative_directory.is_empty());
    let metadata = read_manifest(root)
        .or_else(|| root_skill.map(|skill| skill.metadata.clone()))
        .unwrap_or_else(|| SkillMetadata {
            name: id.to_string(),
            description: format!("包含 {} 个 Skills", parsed.len()),
            author: None,
            version: None,
        });
    let html_content = root_skill
        .map(|skill| skill.html_content.clone())
        .unwrap_or_default();
    let members = parsed
        .into_iter()
        .map(|skill| SkillMember {
            id: member_id(id, &skill.relative_directory),
            relative_path: skill.relative_directory,
            metadata: skill.metadata,
            html_content: skill.html_content,
            custom_description: None,
        })
        .collect();

    Ok(DiscoveredSkill {
        id: id.to_string(),
        kind: SkillKind::Pack,
        metadata,
        html_content,
        members,
        source_path: root.to_path_buf(),
        has_executable_content,
        warnings,
    })
}

fn member_id(pack_id: &str, relative_directory: &str) -> String {
    if relative_directory.is_empty() {
        format!("{pack_id}::root")
    } else {
        format!("{pack_id}::{relative_directory}")
    }
}

fn walk(directory: &Path, markdown_paths: &mut Vec<PathBuf>) -> DomainResult<()> {
    for entry in
        fs::read_dir(directory).map_err(|error| DomainError::Database(error.to_string()))?
    {
        let entry = entry.map_err(|error| DomainError::Database(error.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if file_type.is_dir() {
            if should_ignore_directory(&name) {
                continue;
            }
            walk(&path, markdown_paths)?;
        } else if file_type.is_file() {
            if name == "SKILL.md" {
                markdown_paths.push(path);
            }
        }
    }
    Ok(())
}

fn should_ignore_directory(name: &str) -> bool {
    IGNORED_DIRECTORIES.contains(&name)
        || (name.starts_with('.') && name != ".claude-plugin" && name != ".codex-plugin")
}

fn is_executable_candidate(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            EXECUTABLE_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}

fn directory_has_executable_content(directory: &Path) -> DomainResult<bool> {
    for entry in
        fs::read_dir(directory).map_err(|error| DomainError::Database(error.to_string()))?
    {
        let entry = entry.map_err(|error| DomainError::Database(error.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| DomainError::Database(error.to_string()))?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if file_type.is_dir() {
            if should_ignore_directory(&name) {
                continue;
            }
            if name == "hooks" || directory_has_executable_content(&path)? {
                return Ok(true);
            }
        } else if file_type.is_file() && is_executable_candidate(&path) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn read_manifest(root: &Path) -> Option<SkillMetadata> {
    [
        root.join(".codex-plugin/plugin.json"),
        root.join(".claude-plugin/plugin.json"),
    ]
    .into_iter()
    .find_map(|path| {
        let content = fs::read_to_string(path).ok()?;
        let manifest: PluginManifest = serde_json::from_str(&content).ok()?;
        Some(SkillMetadata {
            description: manifest
                .description
                .unwrap_or_else(|| format!("{} Skill Pack", manifest.name)),
            name: manifest.name,
            author: manifest.author.map(ManifestAuthor::into_name),
            version: manifest.version,
        })
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::scan_skill_root;

    struct Fixture {
        path: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("agentforge-scan-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn skill(&self, relative: &str, name: &str) {
            let directory = self.path.join(relative);
            fs::create_dir_all(&directory).unwrap();
            fs::write(
                directory.join("SKILL.md"),
                format!("---\nname: {name}\ndescription: {name} description\n---\n\n# {name}\n"),
            )
            .unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn file_name(path: &Path) -> String {
        path.file_name().unwrap().to_string_lossy().into_owned()
    }

    #[test]
    fn classifies_nested_skills_as_one_pack() {
        let fixture = Fixture::new();
        fixture.skill("skills/alpha", "Alpha");
        fixture.skill("skills/beta", "Beta");

        let discovered = scan_skill_root("example-pack", &fixture.path).unwrap();

        assert_eq!(discovered.kind.as_str(), "pack");
        assert_eq!(discovered.members.len(), 2);
        assert_eq!(discovered.members[0].metadata.name, "Alpha");
        assert_eq!(discovered.members[1].metadata.name, "Beta");
    }

    #[test]
    fn treats_one_nested_skill_as_standalone() {
        let fixture = Fixture::new();
        fixture.skill("skills/only", "Only");

        let discovered = scan_skill_root("nested", &fixture.path).unwrap();

        assert_eq!(discovered.kind.as_str(), "standalone");
        assert_eq!(discovered.metadata.name, "Only");
        assert!(discovered.members.is_empty());
    }

    #[test]
    fn ignores_dependency_and_git_directories() {
        let fixture = Fixture::new();
        fixture.skill("skills/real", "Real");
        fixture.skill(".git/skills/fake", "Git fake");
        fixture.skill("node_modules/tool", "Dependency fake");

        let discovered = scan_skill_root("safe", &fixture.path).unwrap();

        assert_eq!(discovered.metadata.name, "Real");
        assert_eq!(file_name(&discovered.source_path), "real");
    }

    #[test]
    fn keeps_valid_members_and_reports_malformed_ones() {
        let fixture = Fixture::new();
        fixture.skill("skills/good", "Good");
        let broken = fixture.path.join("skills/broken");
        fs::create_dir_all(&broken).unwrap();
        fs::write(broken.join("SKILL.md"), "not frontmatter").unwrap();

        let discovered = scan_skill_root("mixed", &fixture.path).unwrap();

        assert_eq!(discovered.kind.as_str(), "pack");
        assert_eq!(discovered.metadata.name, "mixed");
        assert_eq!(discovered.members.len(), 1);
        assert_eq!(discovered.warnings.len(), 1);
    }

    #[test]
    fn detects_executable_content() {
        let fixture = Fixture::new();
        fixture.skill("skills/only", "Only");
        fs::create_dir_all(fixture.path.join("skills/only/hooks")).unwrap();
        fs::write(
            fixture.path.join("skills/only/hooks/start.sh"),
            "#!/bin/sh\n",
        )
        .unwrap();

        let discovered = scan_skill_root("scripted", &fixture.path).unwrap();

        assert!(discovered.has_executable_content);
    }

    #[test]
    fn ignores_executable_content_outside_a_nested_standalone_skill() {
        let fixture = Fixture::new();
        fixture.skill("skills/only", "Only");
        fs::create_dir_all(fixture.path.join("examples")).unwrap();
        fs::write(fixture.path.join("examples/demo.py"), "print('demo')\n").unwrap();

        let discovered = scan_skill_root("clean", &fixture.path).unwrap();

        assert!(!discovered.has_executable_content);
    }
}
