import { cleanup, render, screen, within } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { createMemoryRouter, RouterProvider } from 'react-router-dom';
import { afterEach, describe, expect, it, vi } from 'vitest';

const { getHealthMock } = vi.hoisted(() => ({
  getHealthMock: vi.fn().mockResolvedValue({
    version: '0.1.0',
    platform: 'macos',
    database: 'ready',
    ready: true,
  }),
}));

vi.mock('../shared/api/tauriClient', async (importOriginal) => {
  const original = await importOriginal<typeof import('../shared/api/tauriClient')>();
  return { ...original, getHealth: getHealthMock };
});

import { appRoutes } from '../app/router';

describe('foundation routes', () => {
  afterEach(() => {
    cleanup();
  });

  it.each([
    ['/', '控制面板'],
    ['/projects', '项目管理'],
    ['/skills', 'Skills 管理'],
    ['/mcp', 'MCP 管理'],
    ['/tasks', '任务中心'],
    ['/settings', '设置'],
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

    const projectNav = await screen.findByRole('navigation', { name: '项目详情' });
    for (const tab of ['概览', 'Harness', 'Agents', '环境']) {
      expect(within(projectNav).getByRole('link', { name: tab })).toBeInTheDocument();
    }
  });
});
