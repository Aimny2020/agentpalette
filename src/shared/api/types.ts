export type DatabaseStatus = 'ready' | 'unavailable';

export interface HealthReport {
  version: string;
  platform: string;
  database: DatabaseStatus;
  ready: boolean;
}

export interface CommandFailure {
  code: string;
  message: string;
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
