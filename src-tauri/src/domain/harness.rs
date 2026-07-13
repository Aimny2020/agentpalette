#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessTemplateFile {
    pub path: String,
    pub kind: String,
    pub standard: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessPresetFile {
    pub path: String,
    pub kind: String,
    pub label: String,
    pub content: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessPreset {
    pub id: String,
    pub work_type: String,
    pub name: String,
    pub description: String,
    pub files: Vec<HarnessPresetFile>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodeWorkModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub files: Vec<HarnessPresetFile>,
    pub agent_instructions: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct HarnessManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub work_type: String,
    #[serde(default = "default_harness_language")]
    pub language: String,
    #[serde(default)]
    pub created_from_preset: Option<String>,
    #[serde(default)]
    pub selected_modules: Vec<String>,
    pub source: String,
    pub required_files: Vec<String>,
    #[serde(default)]
    pub files: Vec<HarnessTemplateFile>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessTemplateSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub work_type: String,
    pub language: String,
    pub created_from_preset: Option<String>,
    #[serde(default)]
    pub selected_modules: Vec<String>,
    pub source_type: String, // "local" | "project"
    pub source_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub file_count: usize,
    pub has_agents_md: bool,
    pub has_manifest: bool,
    pub is_valid: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessValidationReport {
    pub has_agents_md: bool,
    pub has_manifest: bool,
    pub manifest_parses: bool,
    pub missing_required_files: Vec<String>,
    pub syntax_errors: Vec<String>,
    pub warnings: Vec<String>,
    pub is_valid: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessFileSummary {
    pub path: String,
    pub size: u64,
    pub is_standard: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessTemplateDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub work_type: String,
    pub language: String,
    pub created_from_preset: Option<String>,
    #[serde(default)]
    pub selected_modules: Vec<String>,
    pub source_type: String,
    pub source_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub files: Vec<HarnessFileSummary>,
    pub validation: HarnessValidationReport,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessFile {
    pub path: String,
    pub content: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateHarnessTemplateInput {
    pub name: String,
    pub description: String,
    pub work_type: String,
    #[serde(default = "default_harness_language")]
    pub language: String,
    pub preset_id: Option<String>,
    #[serde(default)]
    pub selected_modules: Vec<String>,
    pub optional_files: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessImportInspection {
    pub has_agents_md: bool,
    pub has_manifest: bool,
    pub name: Option<String>,
    pub description: Option<String>,
    pub work_type: Option<String>,
    pub found_files: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessImportOptions {
    pub name: String,
    pub description: String,
    pub work_type: String,
    #[serde(default = "default_harness_language")]
    pub language: String,
    pub preset_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessExtractOptions {
    pub name: String,
    pub description: String,
    pub work_type: String,
    #[serde(default = "default_harness_language")]
    pub language: String,
    pub preset_id: Option<String>,
    pub selected_files: Vec<String>,
}

pub fn default_harness_language() -> String {
    "en".into()
}
