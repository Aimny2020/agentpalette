import { invoke } from '@tauri-apps/api/core';

import type { CommandFailure, HealthReport, Skill, Category, Project } from './types';

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

export async function deleteSkill(skillId: string): Promise<void> {
  try {
    await invoke<void>('delete_skill', { skillId });
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
