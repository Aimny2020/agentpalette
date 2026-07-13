import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it, vi } from 'vitest';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

import { AppShell } from './AppShell';

const { getProjectsMock } = vi.hoisted(() => ({
  getProjectsMock: vi.fn().mockResolvedValue([
    { id: 'agent-forge-core-id', name: 'Agent-Forge-Core', path: '/users/dev/core', created_at: '' }
  ]),
}));

vi.mock('../../shared/api/tauriClient', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../shared/api/tauriClient')>();
  return {
    ...original,
    getProjects: getProjectsMock,
  };
});

describe('AppShell', () => {
  it('shows global navigation and the selected project', async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
        },
      },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/projects']}>
          <AppShell />
        </MemoryRouter>
      </QueryClientProvider>,
    );

    expect(screen.getByRole('banner')).toBeInTheDocument();
    expect(screen.getByRole('main')).toBeInTheDocument();
    for (const label of ['项目管理', 'Skills', 'Harness', '设置']) {
      expect(screen.getByRole('link', { name: label })).toBeInTheDocument();
    }
    expect(screen.queryByRole('link', { name: '控制面板' })).not.toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'MCP' })).not.toBeInTheDocument();
    expect(screen.queryByRole('link', { name: '任务' })).not.toBeInTheDocument();
    // Wait for the query to resolve and show the project
    expect(await screen.findByText('Agent-Forge-Core')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: '项目管理' })).toHaveAttribute(
      'aria-current',
      'page',
    );
  });
});
