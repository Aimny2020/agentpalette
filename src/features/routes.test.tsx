import { cleanup, render, screen, within } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { createMemoryRouter, RouterProvider } from 'react-router-dom';
import { afterEach, describe, expect, it, vi } from 'vitest';

const { getHealthMock, getLaunchPreferencesMock, getLocalAgentsMock, checkAgentUpdatesMock } = vi.hoisted(() => ({
  getHealthMock: vi.fn().mockResolvedValue({
    version: '0.1.0',
    platform: 'macos',
    database: 'ready',
    ready: true,
  }),
  getLaunchPreferencesMock: vi.fn().mockResolvedValue({
    macosTerminal: 'auto',
    windowsTerminal: 'auto',
    launchPresentation: 'new_tab',
    showCommandPreview: true,
    checkEnvironment: true,
    checkPermissions: true,
    allowCopyCommandFallback: true,
  }),
  getLocalAgentsMock: vi.fn().mockResolvedValue([]),
  checkAgentUpdatesMock: vi.fn().mockResolvedValue([]),
}));

vi.mock('../shared/api/tauriClient', async (importOriginal) => {
  const original = await importOriginal<typeof import('../shared/api/tauriClient')>();
  return { ...original, getHealth: getHealthMock, getLaunchPreferences: getLaunchPreferencesMock, getLocalAgents: getLocalAgentsMock, checkAgentUpdates: checkAgentUpdatesMock };
});

import { appRoutes } from '../app/router';

describe('foundation routes', () => {
  afterEach(() => {
    cleanup();
  });

  it.each([
    ['/', 'Projects'],
    ['/projects', 'Projects'],
    ['/skills', 'Skills'],
    ['/agents', 'Agents'],
    ['/tasks', '任务中心'],
    ['/settings', 'Settings'],
  ])('renders %s as %s', async (path, heading) => {
    const router = createMemoryRouter(appRoutes, { initialEntries: [path] });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={client}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    expect(await screen.findByRole('heading', { name: heading, level: 1 })).toBeInTheDocument();
  });

  it('shows project detail tabs', async () => {
    const router = createMemoryRouter(appRoutes, { initialEntries: ['/projects'] });
    const client = new QueryClient();
    render(
      <QueryClientProvider client={client}>
        <RouterProvider router={router} />
      </QueryClientProvider>,
    );

    const projectNav = await screen.findByRole('navigation', { name: 'Project details' });
    for (const tab of ['Overview', 'Harness', 'Skills']) {
      expect(within(projectNav).getByRole('link', { name: tab })).toBeInTheDocument();
    }
  });
});
