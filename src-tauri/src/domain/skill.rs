use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillKind {
    Standalone,
    Pack,
}

impl SkillKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standalone => "standalone",
            Self::Pack => "pack",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMember {
    pub id: String,
    pub relative_path: String,
    pub metadata: SkillMetadata,
    pub html_content: String,
    pub custom_description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Local,
    Git,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Git => "git",
        }
    }

    pub fn from_db(value: &str) -> Self {
        if value == "git" {
            Self::Git
        } else {
            Self::Local
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStatus {
    NotApplicable,
    Unknown,
    Current,
    Available,
    Dirty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSourceInfo {
    pub kind: SourceKind,
    pub url: Option<String>,
    pub tracked_ref: Option<String>,
    pub installed_commit: Option<String>,
}

impl SkillSourceInfo {
    pub fn local() -> Self {
        Self {
            kind: SourceKind::Local,
            url: None,
            tracked_ref: None,
            installed_commit: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillPackageRecord {
    pub skill_id: String,
    pub source_kind: SourceKind,
    pub source_url: Option<String>,
    pub normalized_source: Option<String>,
    pub tracked_ref: Option<String>,
    pub installed_commit: Option<String>,
    pub trusted_commit: Option<String>,
    pub last_checked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdate {
    pub skill_id: String,
    pub status: UpdateStatus,
    pub installed_commit: Option<String>,
    pub available_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInspection {
    pub name: String,
    pub kind: SkillKind,
    pub member_count: usize,
    pub has_executable_content: bool,
    pub warnings: Vec<String>,
    pub recommended_ref: Option<String>,
    pub duplicate_skill_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub kind: SkillKind,
    pub metadata: SkillMetadata,
    pub html_content: String,
    pub members: Vec<SkillMember>,
    pub category_id: Option<String>,
    pub user_notes: Option<String>,
    pub source: SkillSourceInfo,
    pub update_status: UpdateStatus,
    pub available_commit: Option<String>,
    pub has_executable_content: bool,
    pub trusted: bool,
    pub warnings: Vec<String>,
    pub custom_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSkillMeta {
    pub category_id: Option<String>,
    pub user_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillDescriptionRecord {
    pub target_id: String,
    pub target_kind: String, // 'package' or 'member'
    #[serde(rename = "description")]
    pub custom_description: String,
    pub updated_at: String,
}
