use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHarnessAppliedFile {
    pub path: String,
    pub applied_content_hash: String,
    pub created_by_application: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHarnessRecord {
    pub project_id: String,
    pub source_template_id: Option<String>,
    pub source_template_hash: Option<String>,
    pub applied_at: String,
    pub managed_state: String,
    pub applied_files: Vec<ProjectHarnessAppliedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHarnessFile {
    pub path: String,
    pub content: String,
    pub exists: bool,
    pub changed_since_apply: bool,
    pub deletion_eligible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHarnessStatus {
    pub project_id: String,
    pub state: String,
    pub source_template_id: Option<String>,
    pub source_template_hash: Option<String>,
    pub applied_at: Option<String>,
    pub source_status: String,
    pub has_agents_md: bool,
    pub manifest_parseable: bool,
    pub files: Vec<ProjectHarnessFile>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHarnessConflict {
    pub path: String,
    pub template_content: String,
    pub project_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHarnessApplicationPreview {
    pub project_id: String,
    pub template_id: String,
    pub conflicts: Vec<ProjectHarnessConflict>,
    pub template_files: Vec<ProjectHarnessFile>,
    pub final_agents_references: Vec<String>,
    pub missing_agents_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHarnessFileDecision {
    pub path: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHarnessApplyInput {
    pub project_id: String,
    pub template_id: String,
    pub decisions: Vec<ProjectHarnessFileDecision>,
}
