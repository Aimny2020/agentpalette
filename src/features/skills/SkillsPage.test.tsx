import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, expect, it, vi } from 'vitest';
import { SkillsPage } from './SkillsPage';
import * as tauriClient from '../../shared/api/tauriClient';
import type { Skill } from '../../shared/api/types';

vi.mock('../../shared/api/tauriClient', () => ({
  getSkills: vi.fn(),
  getSkillDetail: vi.fn(),
  getCategories: vi.fn(),
  checkSkillUpdates: vi.fn(),
  trustSkill: vi.fn(),
  updateSkillMeta: vi.fn(),
  updateSkill: vi.fn(),
  deleteSkill: vi.fn(),
  deleteSkillEverywhere: vi.fn(),
  importSkill: vi.fn(),
  createCategory: vi.fn(),
  renameCategory: vi.fn(),
  deleteCategory: vi.fn(),
  saveCustomDescription: vi.fn(),
}));

const mockUntrustedSkill: Skill = {
  id: 'test-skill',
  kind: 'standalone',
  metadata: {
    name: 'Test Skill',
    description: 'Test skill description',
    version: '1.0.0',
  },
  members: [],
  source: { kind: 'local' },
  has_executable_content: true,
  trusted: false,
  update_status: 'unknown',
  warnings: [],
  html_content: '<p>Skill doc</p>',
};

const mockGitSkill: Skill = {
  ...mockUntrustedSkill,
  id: 'git-skill',
  metadata: {
    name: 'Git Skill',
    description: 'A Skill imported from GitHub',
    version: '2.0.0',
  },
  source: {
    kind: 'git',
    url: 'https://github.com/example/git-skill',
    tracked_ref: 'main',
    installed_commit: 'abcdef123',
  },
  update_status: 'current',
  has_executable_content: false,
  trusted: true,
};

describe('SkillsPage', () => {
  it('filters skills by import source', async () => {
    vi.mocked(tauriClient.getSkills).mockResolvedValue([mockUntrustedSkill, mockGitSkill]);
    vi.mocked(tauriClient.getCategories).mockResolvedValue([]);

    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <SkillsPage />
      </QueryClientProvider>
    );

    expect(await screen.findByText('Test Skill')).toBeInTheDocument();
    expect(screen.getByText('Git Skill')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /^Local/ }));
    expect(screen.getByText('Test Skill')).toBeInTheDocument();
    expect(screen.queryByText('Git Skill')).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /^Git/ }));
    expect(screen.getByText('Git Skill')).toBeInTheDocument();
    expect(screen.queryByText('Test Skill')).not.toBeInTheDocument();
  });

  it('uses the designed in-app dialog when a skill is in use', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm');
    vi.mocked(tauriClient.getSkills).mockResolvedValue([mockUntrustedSkill]);
    vi.mocked(tauriClient.getCategories).mockResolvedValue([]);
    vi.mocked(tauriClient.deleteSkill).mockRejectedValue({
      message: '本地数据库暂时不可用，请重试。',
      details: 'Skill Pack is enabled in projects: Project A (/tmp/a)',
    });
    vi.mocked(tauriClient.deleteSkillEverywhere).mockResolvedValue(undefined);

    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    const { container } = render(
      <QueryClientProvider client={queryClient}>
        <SkillsPage />
      </QueryClientProvider>
    );

    await screen.findByText('Test Skill');
    const deleteButton = container.querySelector('.delete-btn');
    expect(deleteButton).not.toBeNull();
    fireEvent.click(deleteButton!);

    fireEvent.click(await screen.findByText('Confirm deletion'));

    expect(await screen.findByText('This Skill is in use')).toBeInTheDocument();
    expect(screen.getByText('Project A (/tmp/a)')).toBeInTheDocument();
    expect(confirmSpy).not.toHaveBeenCalled();

    fireEvent.click(screen.getByText('Remove everywhere and delete'));
    await waitFor(() => {
      expect(tauriClient.deleteSkillEverywhere).toHaveBeenCalledWith(
        'test-skill',
        expect.anything(),
      );
    });

    confirmSpy.mockRestore();
  });

  it('updates detail modal to "已信任此版本" after clicking trust button', async () => {
    vi.mocked(tauriClient.getSkills).mockResolvedValue([mockUntrustedSkill]);
    vi.mocked(tauriClient.getSkillDetail).mockResolvedValue(mockUntrustedSkill);
    vi.mocked(tauriClient.getCategories).mockResolvedValue([]);
    vi.mocked(tauriClient.trustSkill).mockResolvedValue(undefined);

    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <SkillsPage />
      </QueryClientProvider>
    );

    // Wait for skills card to appear
    const cardTitle = await screen.findByText('Test Skill');
    fireEvent.click(cardTitle);

    // Modal should be open and display "信任此版本" button
    const trustButton = await screen.findByRole('button', { name: /信任此版本/ });
    expect(trustButton).toBeInTheDocument();

    // After backend trust, getSkillDetail returns trusted skill
    vi.mocked(tauriClient.getSkillDetail).mockResolvedValue({
      ...mockUntrustedSkill,
      trusted: true,
    });

    fireEvent.click(trustButton);

    // Expect trustSkill to have been called
    await waitFor(() => {
      expect(tauriClient.trustSkill).toHaveBeenCalledWith('test-skill');
    });

    // Button should change to "已信任此版本"
    await waitFor(() => {
      expect(screen.getByText('已信任此版本')).toBeInTheDocument();
    });
    expect(screen.queryByRole('button', { name: /信任此版本/ })).not.toBeInTheDocument();
  });
});
