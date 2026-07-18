import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, expect, it, vi } from 'vitest';
import { AgentsPage } from './AgentsPage';
import * as tauriClient from '../../shared/api/tauriClient';

vi.mock('../../shared/api/tauriClient', () => ({
  getLocalAgents: vi.fn(),
  checkAgentUpdates: vi.fn(),
  getAgentMaintenancePlan: vi.fn(),
  applyAgentMaintenance: vi.fn(),
  openDesktopAgent: vi.fn(),
}));

describe('AgentsPage', () => {
  it('renders local agent version and update status correctly', async () => {
    vi.mocked(tauriClient.getLocalAgents).mockResolvedValue([
      {
        id: 'claude',
        product: 'Claude',
        displayName: 'Claude Code',
        surface: 'cli',
        command: 'claude',
        status: 'ready',
        version: '1.0.5',
        executablePath: 'C:\\bin\\claude.cmd',
        canInstall: false,
        canUpdate: true,
        canUninstall: true,
      },
    ]);
    vi.mocked(tauriClient.checkAgentUpdates).mockResolvedValue([
      {
        agentId: 'claude',
        status: 'current',
        currentVersion: '1.0.5',
        latestVersion: '1.0.5',
      },
    ]);

    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <AgentsPage />
      </QueryClientProvider>
    );

    expect(await screen.findByText(/v1.0.5/)).toBeInTheDocument();
    expect(await screen.findByText('已是最新版本')).toBeInTheDocument();
  });
});
