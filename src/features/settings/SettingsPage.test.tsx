import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createMemoryRouter, Link, RouterProvider } from 'react-router-dom';

const { getHealthMock, getLaunchPreferencesMock, saveLaunchPreferencesMock } = vi.hoisted(() => ({
  getHealthMock: vi.fn(),
  getLaunchPreferencesMock: vi.fn(),
  saveLaunchPreferencesMock: vi.fn(),
}));

vi.mock('../../shared/api/tauriClient', () => ({
  getHealth: getHealthMock,
  getLaunchPreferences: getLaunchPreferencesMock,
  saveLaunchPreferences: saveLaunchPreferencesMock,
}));

import { useThemeStore } from '../../shared/theme/themeStore';
import { SettingsPage } from './SettingsPage';

const preferences = {
  macosTerminal: 'auto' as const,
  windowsTerminal: 'auto' as const,
  launchPresentation: 'new_tab' as const,
  showCommandPreview: true,
  checkEnvironment: true,
  checkPermissions: true,
  allowCopyCommandFallback: true,
};

function renderPage() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const router = createMemoryRouter([
    { path: '/settings', element: <><Link to="/projects">项目管理</Link><SettingsPage /></> },
    { path: '/projects', element: <div><h1>项目</h1><Link to="/settings">设置</Link></div> },
  ], { initialEntries: ['/settings'] });
  render(<QueryClientProvider client={client}><RouterProvider router={router} /></QueryClientProvider>);
  return router;
}

describe('SettingsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useThemeStore.setState({ theme: 'system' });
    getHealthMock.mockResolvedValue({ version: '0.2.1', platform: 'macos', database: 'ready', ready: true });
    getLaunchPreferencesMock.mockResolvedValue(preferences);
    saveLaunchPreferencesMock.mockImplementation(async (value) => value);
  });

  it('opens basic settings without loading platform preferences', async () => {
    const user = userEvent.setup();
    renderPage();

    expect(screen.getByRole('heading', { name: '基础设置', level: 2 })).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: '界面语言' })).toBeDisabled();
    expect(screen.getByRole('combobox', { name: '界面语言' })).toHaveValue('zh-CN');
    expect(getHealthMock).not.toHaveBeenCalled();
    expect(getLaunchPreferencesMock).not.toHaveBeenCalled();

    await user.click(screen.getByRole('radio', { name: /深色/ }));
    expect(useThemeStore.getState().theme).toBe('dark');
  });

  it('shows macOS terminal preferences and saves the selected value', async () => {
    const user = userEvent.setup();
    renderPage();

    await user.click(screen.getByRole('button', { name: '平台启动偏好' }));
    await user.click(await screen.findByRole('radio', { name: /iTerm2/ }));
    await user.click(screen.getByRole('button', { name: '保存启动偏好' }));

    expect(saveLaunchPreferencesMock).toHaveBeenCalledWith(
      { ...preferences, macosTerminal: 'iterm' },
      expect.anything(),
    );
  });

  it('loads version information only when About is selected', async () => {
    const user = userEvent.setup();
    renderPage();

    await user.click(screen.getByRole('button', { name: '关于' }));

    expect(await screen.findByText('0.2.1')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '检查更新（暂未开放）' })).toBeDisabled();
    expect(getLaunchPreferencesMock).not.toHaveBeenCalled();
  });

  it('keeps launch drafts across tabs and prompts before app navigation', async () => {
    const user = userEvent.setup();
    renderPage();

    await user.click(screen.getByRole('button', { name: '平台启动偏好' }));
    await user.click(await screen.findByRole('radio', { name: /iTerm2/ }));
    await user.click(screen.getByRole('button', { name: '基础设置' }));
    await user.click(screen.getByRole('button', { name: '平台启动偏好' }));
    expect(screen.getByRole('radio', { name: /iTerm2/ })).toBeChecked();

    await user.click(screen.getByRole('link', { name: '项目管理' }));
    expect(await screen.findByRole('dialog', { name: '保存启动偏好？' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: '继续编辑' }));

    expect(screen.queryByRole('dialog', { name: '保存启动偏好？' })).not.toBeInTheDocument();
    expect(screen.getByRole('radio', { name: /iTerm2/ })).toBeChecked();
  });
});
