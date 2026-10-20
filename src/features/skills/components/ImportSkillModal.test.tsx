import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { ImportSkillModal } from './ImportSkillModal';
import { AppError, getGitEnvironment, inspectSkillImport } from '../../../shared/api/tauriClient';
import i18n from '../../../shared/i18n/i18n';

vi.mock('../../../shared/api/tauriClient', () => {
  class MockAppError extends Error {
    constructor(
      public readonly code: string,
      message: string,
      public readonly details?: string,
    ) {
      super(message);
      this.name = 'AppError';
    }
  }
  return {
    AppError: MockAppError,
    inspectSkillImport: vi.fn(),
    getGitEnvironment: vi.fn(),
  };
});

function renderModal(props: { onClose: () => void; onImport: () => void }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <ImportSkillModal {...props} />
    </QueryClientProvider>,
  );
}

describe('ImportSkillModal', () => {
  beforeEach(async () => {
    vi.mocked(inspectSkillImport).mockReset();
    vi.mocked(getGitEnvironment).mockReset();
    vi.mocked(getGitEnvironment).mockResolvedValue({
      available: true,
      version: 'git version 2.45.1.windows.1',
      executable: 'C:\\Program Files\\Git\\cmd\\git.exe',
      proxy: 'http://127.0.0.1:7890',
      proxySource: 'system',
      pacDetected: false,
      error: null,
    });
    await i18n.changeLanguage('zh-CN');
  });

  it('renders installation ID and normalized Git source on inspection preview', async () => {
    vi.mocked(inspectSkillImport).mockResolvedValue({
      name: 'Sample Pack',
      kind: 'pack',
      member_count: 3,
      has_executable_content: false,
      warnings: [],
      install_id: 'owner-sample-pack',
      normalized_source: 'github.com/owner/sample-pack',
    });

    const handleClose = vi.fn();
    const handleImport = vi.fn();

    renderModal({ onClose: handleClose, onImport: handleImport });

    fireEvent.click(screen.getByText('Git 仓库导入'));

    const input = screen.getByPlaceholderText('https://github.com/org/my-skill-repo');
    fireEvent.change(input, { target: { value: 'https://github.com/owner/sample-pack.git' } });

    const checkBtn = screen.getByText('检查内容');
    await act(async () => {
      fireEvent.click(checkBtn);
    });

    expect(inspectSkillImport).toHaveBeenCalledWith('https://github.com/owner/sample-pack.git', 'git');

    expect(screen.getByText('Sample Pack')).toBeInTheDocument();
    expect(screen.getByText('安装 ID：owner-sample-pack')).toBeInTheDocument();
    expect(screen.getByText('Git 来源：github.com/owner/sample-pack')).toBeInTheDocument();
    expect(screen.getByText('技能扩展包 · 3 个 Skills')).toBeInTheDocument();

    const submitBtn = screen.getByText('确认导入');
    expect(submitBtn).toBeEnabled();

    await act(async () => {
      fireEvent.click(submitBtn);
    });
    expect(handleImport).toHaveBeenCalledWith('https://github.com/owner/sample-pack.git', 'git');
    expect(handleClose).toHaveBeenCalled();
  });

  it('shows the active proxy in the Git import form', async () => {
    renderModal({ onClose: vi.fn(), onImport: vi.fn() });

    fireEvent.click(screen.getByText('Git 仓库导入'));

    expect(await screen.findByText('代理：http://127.0.0.1:7890（系统代理）')).toBeInTheDocument();
  });

  it('surfaces the detailed Git failure from the backend', async () => {
    vi.mocked(inspectSkillImport).mockRejectedValue(
      new AppError(
        'git_unavailable',
        'Git 操作失败，请检查 Git 安装与网络代理设置。',
        'fatal: unable to access repository: Recv failure: Connection was reset',
      ),
    );

    renderModal({ onClose: vi.fn(), onImport: vi.fn() });

    fireEvent.click(screen.getByText('Git 仓库导入'));
    fireEvent.change(screen.getByPlaceholderText('https://github.com/org/my-skill-repo'), {
      target: { value: 'https://github.com/owner/sample-pack.git' },
    });

    await act(async () => {
      fireEvent.click(screen.getByText('检查内容'));
    });

    await waitFor(() => {
      expect(screen.getByText(/Recv failure: Connection was reset/)).toBeInTheDocument();
    });
    expect(screen.getByText(/Git 操作失败/)).toBeInTheDocument();
  });

  it('disables confirmation button and shows duplicate warning when source is duplicate', async () => {
    vi.mocked(inspectSkillImport).mockResolvedValue({
      name: 'Sample Standalone',
      kind: 'standalone',
      member_count: 1,
      has_executable_content: false,
      warnings: [],
      install_id: 'owner-sample-standalone',
      normalized_source: 'github.com/owner/sample-standalone',
      duplicate_skill_id: 'owner-sample-standalone',
    });

    renderModal({ onClose: vi.fn(), onImport: vi.fn() });

    fireEvent.click(screen.getByText('Git 仓库导入'));

    fireEvent.change(screen.getByPlaceholderText('https://github.com/org/my-skill-repo'), {
      target: { value: 'https://github.com/owner/sample-standalone.git' },
    });

    await act(async () => {
      fireEvent.click(screen.getByText('检查内容'));
    });

    expect(screen.getByText('已安装为 owner-sample-standalone，不会创建重复副本。')).toBeInTheDocument();
    expect(screen.getByText('确认导入')).toBeDisabled();
  });

  it('does not show warning when different repo name has same base name but different source', async () => {
    vi.mocked(inspectSkillImport).mockResolvedValue({
      name: 'Different Repo Name',
      kind: 'standalone',
      member_count: 1,
      has_executable_content: false,
      warnings: [],
      install_id: 'another-skills',
      normalized_source: 'github.com/another/skills',
    });

    renderModal({ onClose: vi.fn(), onImport: vi.fn() });

    fireEvent.click(screen.getByText('Git 仓库导入'));

    fireEvent.change(screen.getByPlaceholderText('https://github.com/org/my-skill-repo'), {
      target: { value: 'https://github.com/another/skills.git' },
    });

    await act(async () => {
      fireEvent.click(screen.getByText('检查内容'));
    });

    expect(screen.queryByText(/已安装为/)).not.toBeInTheDocument();
    expect(screen.getByText('确认导入')).toBeEnabled();
  });
});
