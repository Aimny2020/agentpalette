export type DatabaseStatus = 'ready' | 'unavailable';

export interface HealthReport {
  version: string;
  platform: string;
  database: DatabaseStatus;
  ready: boolean;
}

export type TerminalPreference =
  | 'auto'
  | 'terminal'
  | 'iterm'
  | 'warp'
  | 'vscode'
  | 'windows_terminal'
  | 'powershell'
  | 'git_bash';

export type LaunchPresentation = 'new_tab' | 'new_window';

export interface LaunchPreferences {
  macosTerminal: TerminalPreference;
  windowsTerminal: TerminalPreference;
  launchPresentation: LaunchPresentation;
  showCommandPreview: boolean;
  checkEnvironment: boolean;
  checkPermissions: boolean;
  allowCopyCommandFallback: boolean;
}

export type GitProxyMode = 'system' | 'direct' | 'manual';

export interface NetworkPreferences {
  gitProxyMode: GitProxyMode;
  gitProxyUrl: string;
}

export type GitProxySource = 'manual' | 'environment' | 'system' | 'none' | 'pac';

export interface GitEnvironmentReport {
  available: boolean;
  version: string | null;
  executable: string | null;
  proxy: string | null;
  proxySource: GitProxySource;
  pacDetected: boolean;
  error: string | null;
}

export interface GitConnectionReport {
  success: boolean;
  elapsedMs: number;
  proxy: string | null;
  message: string | null;
}

export type LocalAgentStatus = 'ready' | 'missing';

export interface LocalAgent {
  id: string;
  product: string;
  displayName: string;
  surface: 'cli' | 'desktop';
  command: string;
  status: LocalAgentStatus;
  version?: string;
  executablePath?: string;
  canInstall: boolean;
  canUpdate: boolean;
  canUninstall: boolean;
}

export type AgentUpdateStatus = 'not_installed' | 'unknown' | 'current' | 'available';
export type AgentMaintenanceAction = 'install' | 'update' | 'uninstall';

export interface AgentUpdate {
  agentId: string;
  status: AgentUpdateStatus;
  currentVersion?: string;
  latestVersion?: string;
}

export interface AgentMaintenancePlan {
  agentId: string;
  action: AgentMaintenanceAction;
  command: string;
}

export interface CommandFailure {
  code: string;
  details?: string;
}

export interface SkillMetadata {
  name: string;
  description: string;
  author?: string;
  version?: string;
}

export type SkillKind = 'standalone' | 'pack';
export type SkillSourceKind = 'local' | 'git';
export type SkillUpdateStatus = 'not_applicable' | 'unknown' | 'current' | 'available' | 'dirty';

export interface SkillMember {
  id: string;
  relative_path: string;
  metadata: SkillMetadata;
  html_content: string;
  custom_description?: string;
}

export interface SkillSourceInfo {
  kind: SkillSourceKind;
  url?: string;
  tracked_ref?: string;
  installed_commit?: string;
  last_checked_at?: string;
}

export interface Skill {
  id: string;
  kind: SkillKind;
  metadata: SkillMetadata;
  html_content: string;
  members: SkillMember[];
  category_id?: string;
  user_notes?: string;
  source: SkillSourceInfo;
  update_status: SkillUpdateStatus;
  available_commit?: string;
  has_executable_content: boolean;
  trusted: boolean;
  warnings: string[];
  custom_description?: string;
}

export interface SkillUpdate {
  skill_id: string;
  status: SkillUpdateStatus;
  installed_commit?: string;
  available_commit?: string;
}

export interface ImportInspection {
  name: string;
  kind: SkillKind;
  member_count: number;
  has_executable_content: boolean;
  warnings: string[];
  recommended_ref?: string;
  duplicate_skill_id?: string;
  install_id: string;
  normalized_source?: string;
}

export interface Category {
  id: string;
  name: string;
  created_at: string;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  created_at: string;
}

export interface SkillDescriptionRecord {
  target_id: string;
  target_kind: 'package' | 'member';
  description: string;
  updated_at: string;
}

export interface InvalidRecordInfo {
  target_id?: string;
  target_kind?: string;
  description?: string;
  reason: string;
}

export interface DescriptionsImportPreview {
  file_path: string;
  total_count: number;
  new_count: number;
  overwrite_count: number;
  skip_count: number;
  unassociated_count: number;
  invalid_records: InvalidRecordInfo[];
  valid_records: SkillDescriptionRecord[];
}

export interface HarnessTemplateFile {
  path: string;
  kind: string;
  standard: boolean;
}

export interface HarnessPresetFile {
  path: string;
  kind: string;
  label: string;
  content: string;
}

export interface HarnessPreset {
  id: string;
  workType: string;
  name: string;
  description: string;
  files: HarnessPresetFile[];
}

export interface CodeWorkModule {
  id: string;
  name: string;
  description: string;
  files: HarnessPresetFile[];
  agentInstructions: string;
}

export interface HarnessManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  workType: string;
  language: 'zh-CN' | 'en';
  createdFromPreset?: string;
  selectedModules: string[];
  source: string;
  requiredFiles: string[];
  files: HarnessTemplateFile[];
}

export interface HarnessTemplateSummary {
  id: string;
  name: string;
  description: string;
  workType: string;
  language: 'zh-CN' | 'en';
  createdFromPreset?: string;
  selectedModules: string[];
  sourceType: string;
  sourcePath?: string;
  createdAt: string;
  updatedAt: string;
  fileCount: number;
  hasAgentsMd: boolean;
  hasManifest: boolean;
  isValid: boolean;
}

export interface HarnessValidationReport {
  hasAgentsMd: boolean;
  hasManifest: boolean;
  manifestParses: boolean;
  missingRequiredFiles: string[];
  syntaxErrors: string[];
  warnings: string[];
  isValid: boolean;
}

export interface HarnessFileSummary {
  path: string;
  size: number;
  isStandard: boolean;
}

export interface HarnessTemplateDetail {
  id: string;
  name: string;
  description: string;
  workType: string;
  language: 'zh-CN' | 'en';
  createdFromPreset?: string;
  selectedModules: string[];
  sourceType: string;
  sourcePath?: string;
  createdAt: string;
  updatedAt: string;
  files: HarnessFileSummary[];
  validation: HarnessValidationReport;
}

export interface HarnessFile {
  path: string;
  content: string;
}

export interface CreateHarnessTemplateInput {
  name: string;
  description: string;
  workType: string;
  language: 'zh-CN' | 'en';
  presetId?: string;
  selectedModules: string[];
  optionalFiles: string[];
}

export interface HarnessImportInspection {
  hasAgentsMd: boolean;
  hasManifest: boolean;
  name?: string;
  description?: string;
  workType?: string;
  foundFiles: string[];
}

export interface HarnessImportOptions {
  name: string;
  description: string;
  workType: string;
  language: 'zh-CN' | 'en';
  presetId?: string;
}

export interface HarnessExtractOptions {
  name: string;
  description: string;
  workType: string;
  language: 'zh-CN' | 'en';
  presetId?: string;
  selectedFiles: string[];
}

export interface ProjectHarnessAppliedFile {
  path: string;
  appliedContentHash: string;
  createdByApplication: boolean;
}

export interface ProjectHarnessFile {
  path: string;
  content: string;
  exists: boolean;
  changedSinceApply: boolean;
  deletionEligible: boolean;
}

export interface ProjectHarnessStatus {
  projectId: string;
  state: 'absent' | 'unmanaged_detected' | 'managed' | 'invalid';
  sourceTemplateId?: string;
  sourceTemplateHash?: string;
  appliedAt?: string;
  sourceStatus: 'unknown' | 'available' | 'changed' | 'deleted';
  hasAgentsMd: boolean;
  manifestParseable: boolean;
  files: ProjectHarnessFile[];
  warnings: string[];
}

export interface ProjectHarnessConflict {
  path: string;
  templateContent: string;
  projectContent?: string;
}

export interface ProjectHarnessApplicationPreview {
  projectId: string;
  templateId: string;
  conflicts: ProjectHarnessConflict[];
  templateFiles: ProjectHarnessFile[];
  finalAgentsReferences: string[];
  missingAgentsReferences: string[];
}

export interface ProjectHarnessFileDecision {
  path: string;
  action: 'keep' | 'overwrite' | 'skip';
}

export interface ProjectHarnessApplyInput {
  projectId: string;
  templateId: string;
  decisions: ProjectHarnessFileDecision[];
}
