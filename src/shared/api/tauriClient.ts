import { invoke } from '@tauri-apps/api/core';

import type {
  CommandFailure,
  HealthReport,
  ImportInspection,
  Skill,
  SkillUpdate,
  Category,
  Project,
  SkillDescriptionRecord,
  DescriptionsImportPreview,
  HarnessTemplateSummary,
  HarnessImportInspection,
  HarnessImportOptions,
  HarnessExtractOptions,
  CreateHarnessTemplateInput,
  HarnessTemplateDetail,
  HarnessFile,
  HarnessValidationReport,
} from './types';

export class AppError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly details?: string,
  ) {
    super(message);
    this.name = 'AppError';
  }
}

function isCommandFailure(value: unknown): value is CommandFailure {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    typeof value.code === 'string' &&
    'message' in value &&
    typeof value.message === 'string'
  );
}

function normalizeError(error: unknown): AppError {
  if (isCommandFailure(error)) {
    return new AppError(
      error.code,
      error.message,
      typeof error.details === 'string' ? error.details : undefined,
    );
  }

  return new AppError('unknown_error', '发生未知错误，请重试。');
}

export async function getHealth(): Promise<HealthReport> {
  try {
    return await invoke<HealthReport>('health_check');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function getSkills(): Promise<Skill[]> {
  try {
    return await invoke<Skill[]>('get_skills');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function importSkill(source: string, importType: 'folder' | 'git'): Promise<string> {
  try {
    return await invoke<string>('import_skill', { source, importType });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function inspectSkillImport(
  source: string,
  importType: 'folder' | 'git',
): Promise<ImportInspection> {
  try {
    return await invoke<ImportInspection>('inspect_skill_import', { source, importType });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function deleteSkill(skillId: string): Promise<void> {
  try {
    await invoke<void>('delete_skill', { skillId });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function deleteSkillEverywhere(skillId: string): Promise<void> {
  try {
    await invoke<void>('delete_skill_everywhere', { skillId });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function checkSkillUpdates(): Promise<SkillUpdate[]> {
  try {
    return await invoke<SkillUpdate[]>('check_skill_updates');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function updateSkill(skillId: string): Promise<SkillUpdate> {
  try {
    return await invoke<SkillUpdate>('update_skill', { skillId });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function trustSkill(skillId: string): Promise<void> {
  try {
    await invoke<void>('trust_skill', { skillId });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function updateSkillMeta(
  skillId: string,
  categoryId: string | null,
  userNotes: string | null,
): Promise<void> {
  try {
    await invoke<void>('update_skill_meta', { skillId, categoryId, userNotes });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function getProjectSkills(projectId: string): Promise<string[]> {
  try {
    return await invoke<string[]>('get_project_skills', { projectId });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function toggleProjectSkill(
  projectId: string,
  skillId: string,
  enabled: boolean,
): Promise<void> {
  try {
    await invoke<void>('toggle_project_skill', { projectId, skillId, enabled });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function getCategories(): Promise<Category[]> {
  try {
    return await invoke<Category[]>('get_categories');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function createCategory(name: string): Promise<Category> {
  try {
    return await invoke<Category>('create_category', { name });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function renameCategory(id: string, name: string): Promise<void> {
  try {
    await invoke<void>('rename_category', { id, name });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function deleteCategory(id: string): Promise<void> {
  try {
    await invoke<void>('delete_category', { id });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function getProjects(): Promise<Project[]> {
  try {
    return await invoke<Project[]>('get_projects');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function addProject(path: string): Promise<Project> {
  try {
    return await invoke<Project>('add_project', { path });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function selectDirectory(): Promise<string | null> {
  try {
    return await invoke<string | null>('select_directory');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function deleteProject(id: string): Promise<void> {
  try {
    await invoke<void>('delete_project', { id });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function saveCustomDescription(
  targetId: string,
  targetKind: 'package' | 'member',
  description: string | null,
): Promise<void> {
  try {
    await invoke<void>('save_custom_description', { targetId, targetKind, description });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function exportCustomDescriptions(): Promise<string | null> {
  try {
    return await invoke<string | null>('export_custom_descriptions');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function previewCustomDescriptionsImport(): Promise<DescriptionsImportPreview | null> {
  try {
    return await invoke<DescriptionsImportPreview | null>('preview_custom_descriptions_import');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function confirmCustomDescriptionsImport(
  records: SkillDescriptionRecord[],
  conflictStrategy: 'keep_newer' | 'keep_local' | 'keep_import',
): Promise<void> {
  try {
    await invoke<void>('confirm_custom_descriptions_import', { records, conflictStrategy });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function getUnassociatedDescriptionsCount(): Promise<number> {
  try {
    return await invoke<number>('get_unassociated_descriptions_count');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function clearUnassociatedDescriptions(): Promise<number> {
  try {
    return await invoke<number>('clear_unassociated_descriptions');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function getHarnessTemplates(): Promise<HarnessTemplateSummary[]> {
  try {
    return await invoke<HarnessTemplateSummary[]>('get_harness_templates');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function inspectHarnessImport(sourcePath: string): Promise<HarnessImportInspection> {
  try {
    return await invoke<HarnessImportInspection>('inspect_harness_import', { sourcePath });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function importHarnessFromFolder(
  sourcePath: string,
  options: HarnessImportOptions,
): Promise<HarnessTemplateDetail> {
  try {
    return await invoke<HarnessTemplateDetail>('import_harness_from_folder', { sourcePath, options });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function extractHarnessFromProject(
  projectId: string,
  options: HarnessExtractOptions,
): Promise<HarnessTemplateDetail> {
  try {
    return await invoke<HarnessTemplateDetail>('extract_harness_from_project', { projectId, options });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function createHarnessTemplate(
  input: CreateHarnessTemplateInput,
): Promise<HarnessTemplateDetail> {
  try {
    return await invoke<HarnessTemplateDetail>('create_harness_template', { input });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function getHarnessTemplate(templateId: string): Promise<HarnessTemplateDetail> {
  try {
    return await invoke<HarnessTemplateDetail>('get_harness_template', { templateId });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function readHarnessFile(templateId: string, path: string): Promise<HarnessFile> {
  try {
    return await invoke<HarnessFile>('read_harness_file', { templateId, path });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function writeHarnessFile(
  templateId: string,
  path: string,
  content: string,
): Promise<HarnessFile> {
  try {
    return await invoke<HarnessFile>('write_harness_file', { templateId, path, content });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function createHarnessFile(
  templateId: string,
  path: string,
  kind: string,
): Promise<HarnessFile> {
  try {
    return await invoke<HarnessFile>('create_harness_file', { templateId, path, kind });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function deleteHarnessFile(templateId: string, path: string): Promise<void> {
  try {
    await invoke<void>('delete_harness_file', { templateId, path });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function deleteHarnessTemplate(templateId: string): Promise<void> {
  try {
    await invoke<void>('delete_harness_template', { templateId });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function validateHarnessTemplate(
  templateId: string,
): Promise<HarnessValidationReport> {
  try {
    return await invoke<HarnessValidationReport>('validate_harness_template', { templateId });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function duplicateHarnessTemplate(
  templateId: string,
  targetId: string,
  targetName: string,
): Promise<HarnessTemplateDetail> {
  try {
    return await invoke<HarnessTemplateDetail>('duplicate_harness_template', { templateId, targetId, targetName });
  } catch (error) {
    throw normalizeError(error);
  }
}
