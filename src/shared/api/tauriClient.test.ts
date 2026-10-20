import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  AppError,
  checkSkillUpdates,
  getCodeWorkModules,
  getCodeWorkSharedFiles,
  getHarnessPresets,
  getGitEnvironment,
  getHealth,
  getLaunchPreferences,
  getLocalAgents,
  getNetworkPreferences,
  inspectSkillImport,
  saveNetworkPreferences,
  testGitConnection,
  trustSkill,
  updateSkill,
  saveLaunchPreferences,
  launchAgent,
  openDesktopAgent,
  checkAgentUpdates,
  getAgentMaintenancePlan,
  applyAgentMaintenance,
} from './tauriClient';
import i18n from '../i18n/i18n';

describe('tauriClient', () => {
  beforeEach(async () => {
    invokeMock.mockReset();
    await i18n.changeLanguage('zh-CN');
  });

  it('returns a typed health report', async () => {
    invokeMock.mockResolvedValue({
      version: '0.1.0',
      platform: 'macos',
      database: 'ready',
      ready: true,
    });

    await expect(getHealth()).resolves.toEqual({
      version: '0.1.0',
      platform: 'macos',
      database: 'ready',
      ready: true,
    });
    expect(invokeMock).toHaveBeenCalledWith('health_check');
  });

  it('maps command failures to AppError', async () => {
    invokeMock.mockRejectedValue({
      code: 'database_unavailable',
      details: 'file is locked',
    });

    const failure = await getHealth().catch((error: unknown) => error);

    expect(failure).toBeInstanceOf(AppError);
    expect(failure).toMatchObject({
      code: 'database_unavailable',
      message: '本地数据库暂时不可用，请重试。',
      details: 'file is locked',
    });
  });

  it('uses package lifecycle command payloads', async () => {
    invokeMock.mockResolvedValue([]);
    await checkSkillUpdates();
    expect(invokeMock).toHaveBeenLastCalledWith('check_skill_updates');

    invokeMock.mockResolvedValue({ skill_id: 'pack' });
    await updateSkill('pack');
    expect(invokeMock).toHaveBeenLastCalledWith('update_skill', { skillId: 'pack' });

    invokeMock.mockResolvedValue(undefined);
    await trustSkill('pack');
    expect(invokeMock).toHaveBeenLastCalledWith('trust_skill', { skillId: 'pack' });

    invokeMock.mockResolvedValue({ name: 'Pack', member_count: 5 });
    await inspectSkillImport('https://github.com/example/pack', 'git');
    expect(invokeMock).toHaveBeenLastCalledWith('inspect_skill_import', {
      source: 'https://github.com/example/pack',
      importType: 'git',
    });
  });

  it('uses network preference and Git environment commands', async () => {
    invokeMock.mockResolvedValue({ gitProxyMode: 'system', gitProxyUrl: '' });
    await expect(getNetworkPreferences()).resolves.toEqual({ gitProxyMode: 'system', gitProxyUrl: '' });
    expect(invokeMock).toHaveBeenLastCalledWith('get_network_preferences');

    const preferences = { gitProxyMode: 'manual' as const, gitProxyUrl: 'http://127.0.0.1:7890' };
    await saveNetworkPreferences(preferences);
    expect(invokeMock).toHaveBeenLastCalledWith('save_network_preferences', { preferences });

    invokeMock.mockResolvedValue({ available: true });
    await getGitEnvironment();
    expect(invokeMock).toHaveBeenLastCalledWith('get_git_environment');

    invokeMock.mockResolvedValue({ success: true, elapsedMs: 12 });
    await testGitConnection();
    expect(invokeMock).toHaveBeenLastCalledWith('test_git_connection');
  });

  it('loads the backend-owned Harness preset registry', async () => {
    invokeMock.mockResolvedValue([]);

    await expect(getHarnessPresets()).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenCalledWith('get_harness_presets');
  });

  it('loads the backend-owned Code Work module registry', async () => {
    invokeMock.mockResolvedValue([]);

    await expect(getCodeWorkModules()).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenCalledWith('get_code_work_modules');
  });

  it('loads the backend-owned Code Work shared file library', async () => {
    invokeMock.mockResolvedValue([]);

    await expect(getCodeWorkSharedFiles()).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenCalledWith('get_code_work_shared_files');
  });

  it('loads and saves platform launch preferences with a typed payload', async () => {
    const preferences = {
      macosTerminal: 'iterm' as const,
      windowsTerminal: 'windows_terminal' as const,
      launchPresentation: 'new_tab' as const,
      showCommandPreview: true,
      checkEnvironment: true,
      checkPermissions: true,
      allowCopyCommandFallback: true,
    };
    invokeMock.mockResolvedValue(preferences);

    await expect(getLaunchPreferences()).resolves.toEqual(preferences);
    expect(invokeMock).toHaveBeenLastCalledWith('get_launch_preferences');

    await expect(saveLaunchPreferences(preferences)).resolves.toEqual(preferences);
    expect(invokeMock).toHaveBeenLastCalledWith('save_launch_preferences', { preferences });
  });

  it('loads local agents and launches a selected agent in a project', async () => {
    invokeMock.mockResolvedValue([]);
    await expect(getLocalAgents()).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenLastCalledWith('get_local_agents');

    invokeMock.mockResolvedValue(undefined);
    await expect(launchAgent('project-1', 'codex')).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith('launch_agent', { projectId: 'project-1', agentId: 'codex' });

    await expect(openDesktopAgent('antigravity-desktop')).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith('open_desktop_agent', { agentId: 'antigravity-desktop' });

    invokeMock.mockResolvedValue([]);
    await expect(checkAgentUpdates()).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenLastCalledWith('check_agent_updates');

    invokeMock.mockResolvedValue({ agentId: 'codex', action: 'update', command: 'npm install -g @openai/codex@latest' });
    await expect(getAgentMaintenancePlan('codex', 'update')).resolves.toMatchObject({ agentId: 'codex' });
    expect(invokeMock).toHaveBeenLastCalledWith('get_agent_maintenance_plan', { agentId: 'codex', action: 'update' });

    invokeMock.mockResolvedValue(undefined);
    await expect(applyAgentMaintenance('codex', 'uninstall')).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith('apply_agent_maintenance', { agentId: 'codex', action: 'uninstall' });
  });
});
